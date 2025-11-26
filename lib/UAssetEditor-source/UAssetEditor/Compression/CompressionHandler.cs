using UAssetEditor.Unreal.Readers;

namespace UAssetEditor.Compression;

public static class CompressionHandler
{
    public static byte[] HandleDecompression(UnrealFileReader reader, int compressionIndex, byte[] data, int uncompressedSize)
    {
        var compressionMethod = reader.CompressionMethods[compressionIndex];

        return compressionMethod switch
        {
            "None" => data,
            "Oodle" => Oodle.Decompress(data, uncompressedSize),
            _ => throw new NotImplementedException($"'{compressionMethod}' is not implemented!")
        };
    }
}