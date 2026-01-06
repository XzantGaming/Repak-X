using System.Buffers.Binary;
using ZstdSharp;

/// <summary>
/// Decompresses a ZStandard-compressed .usmap file to an uncompressed format
/// that is compatible with tools like UAssetEditor.
/// </summary>
class Program
{
    const ushort USMAP_MAGIC = 0x30C4;

    enum EUsmapCompressionMethod : byte
    {
        None = 0,
        Oodle = 1,
        Brotli = 2,
        ZStandard = 3
    }

    static int Main(string[] args)
    {
        if (args.Length < 1)
        {
            Console.WriteLine("UsmapDecompressor - Converts compressed .usmap files to uncompressed format");
            Console.WriteLine();
            Console.WriteLine("Usage: UsmapDecompressor <input.usmap> [output.usmap]");
            Console.WriteLine();
            Console.WriteLine("If output is not specified, it will be named <input>_uncompressed.usmap");
            Console.WriteLine();
            Console.WriteLine("Supported compression methods: ZStandard, Brotli");
            return 1;
        }

        string inputPath = args[0];
        string outputPath = args.Length > 1 
            ? args[1] 
            : Path.Combine(
                Path.GetDirectoryName(inputPath) ?? ".",
                Path.GetFileNameWithoutExtension(inputPath) + "_uncompressed.usmap");

        if (!File.Exists(inputPath))
        {
            Console.Error.WriteLine($"Error: Input file not found: {inputPath}");
            return 1;
        }

        try
        {
            byte[] inputData = File.ReadAllBytes(inputPath);
            using var reader = new BinaryReader(new MemoryStream(inputData));

            // Read magic
            ushort magic = reader.ReadUInt16();
            if (magic != USMAP_MAGIC)
            {
                Console.Error.WriteLine($"Error: Invalid .usmap magic: 0x{magic:X4}, expected 0x{USMAP_MAGIC:X4}");
                return 1;
            }

            // Read version
            byte version = reader.ReadByte();
            Console.WriteLine($"Usmap version: {version}");

            // Check for versioning info (version >= 1)
            bool hasVersioning = false;
            int versioningSize = 0;
            long versioningStart = reader.BaseStream.Position;
            
            if (version >= 1)
            {
                hasVersioning = reader.ReadBoolean();
                if (hasVersioning)
                {
                    // Skip FPackageFileVersion (2 ints)
                    reader.ReadInt32();
                    reader.ReadInt32();
                    
                    // Skip FCustomVersionContainer
                    int versionsLength = reader.ReadInt32();
                    reader.BaseStream.Position += versionsLength * (16 + 4); // FGuid + int
                }
            }
            versioningSize = (int)(reader.BaseStream.Position - versioningStart);

            // Some usmap files have extra padding bytes before compression method
            // Check if we're at a valid compression method, if not, skip padding
            long preCompressionPos = reader.BaseStream.Position;
            var compressionMethod = (EUsmapCompressionMethod)reader.ReadByte();
            int compressedSize = reader.ReadInt32();
            int uncompressedSize = reader.ReadInt32();
            
            // Validate - if sizes look wrong, try alternative offset (with padding)
            if (compressedSize < 0 || uncompressedSize < 0 || 
                compressedSize > inputData.Length * 10 || uncompressedSize > inputData.Length * 100)
            {
                Console.WriteLine("Detected padding bytes, adjusting offset...");
                reader.BaseStream.Position = preCompressionPos;
                
                // Skip padding bytes until we find a valid compression method
                // Look for compression method followed by reasonable sizes
                for (int skip = 0; skip < 8; skip++)
                {
                    reader.BaseStream.Position = preCompressionPos + skip;
                    byte testComp = reader.ReadByte();
                    int testCompSize = reader.ReadInt32();
                    int testUncompSize = reader.ReadInt32();
                    
                    if (testComp <= 3 && testCompSize > 0 && testUncompSize > 0 &&
                        testCompSize <= inputData.Length && testUncompSize < inputData.Length * 100)
                    {
                        compressionMethod = (EUsmapCompressionMethod)testComp;
                        compressedSize = testCompSize;
                        uncompressedSize = testUncompSize;
                        Console.WriteLine($"Found valid header at offset +{skip}");
                        break;
                    }
                }
            }

            Console.WriteLine($"Compression: {compressionMethod}");
            Console.WriteLine($"Compressed size: {compressedSize}");
            Console.WriteLine($"Uncompressed size: {uncompressedSize}");

            if (compressionMethod == EUsmapCompressionMethod.None)
            {
                Console.WriteLine("File is already uncompressed. Copying as-is.");
                File.Copy(inputPath, outputPath, true);
                Console.WriteLine($"Output: {outputPath}");
                return 0;
            }

            // Read compressed data
            byte[] compressedData = reader.ReadBytes(compressedSize);
            byte[] uncompressedData;

            // Decompress based on method
            switch (compressionMethod)
            {
                case EUsmapCompressionMethod.ZStandard:
                    using (var decompressor = new Decompressor())
                    {
                        uncompressedData = decompressor.Unwrap(compressedData).ToArray();
                    }
                    break;

                case EUsmapCompressionMethod.Brotli:
                    uncompressedData = new byte[uncompressedSize];
                    using (var brotliStream = new System.IO.Compression.BrotliStream(
                        new MemoryStream(compressedData), 
                        System.IO.Compression.CompressionMode.Decompress))
                    {
                        int totalRead = 0;
                        while (totalRead < uncompressedSize)
                        {
                            int read = brotliStream.Read(uncompressedData, totalRead, uncompressedSize - totalRead);
                            if (read == 0) break;
                            totalRead += read;
                        }
                    }
                    break;

                case EUsmapCompressionMethod.Oodle:
                    Console.Error.WriteLine("Error: Oodle compression is not supported by this tool.");
                    Console.Error.WriteLine("Please use a tool with Oodle support or re-dump with ZStandard/None compression.");
                    return 1;

                default:
                    Console.Error.WriteLine($"Error: Unknown compression method: {compressionMethod}");
                    return 1;
            }

            if (uncompressedData.Length != uncompressedSize)
            {
                Console.Error.WriteLine($"Warning: Decompressed size mismatch. Expected {uncompressedSize}, got {uncompressedData.Length}");
            }

            // Write uncompressed usmap - copy header up to compression method, then write None + sizes + data
            using var output = new BinaryWriter(File.Create(outputPath));
            
            // Copy everything from start up to (but not including) the compression method byte
            // The compression method was found at preCompressionPos + paddingSkip
            int headerEndBeforeCompression = (int)preCompressionPos;
            
            // Find where we actually found the valid compression header
            // We need to figure out how many padding bytes were skipped
            int paddingSkip = 0;
            reader.BaseStream.Position = preCompressionPos;
            for (int skip = 0; skip < 8; skip++)
            {
                reader.BaseStream.Position = preCompressionPos + skip;
                byte testComp = reader.ReadByte();
                int testCompSize = reader.ReadInt32();
                int testUncompSize = reader.ReadInt32();
                
                if (testComp <= 3 && testCompSize > 0 && testUncompSize > 0 &&
                    testCompSize <= inputData.Length && testUncompSize < inputData.Length * 100)
                {
                    paddingSkip = skip;
                    break;
                }
            }
            
            // Write header bytes from start to compression method position (including any padding)
            output.Write(inputData, 0, headerEndBeforeCompression + paddingSkip);
            
            // Write compression method (None)
            output.Write((byte)EUsmapCompressionMethod.None);
            
            // Write sizes (both same for uncompressed)
            output.Write(uncompressedSize);
            output.Write(uncompressedSize);
            
            // Write uncompressed data
            output.Write(uncompressedData);

            Console.WriteLine($"Successfully decompressed to: {outputPath}");
            Console.WriteLine($"Output size: {new FileInfo(outputPath).Length} bytes");
            
            return 0;
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Error: {ex.Message}");
            return 1;
        }
    }
}
