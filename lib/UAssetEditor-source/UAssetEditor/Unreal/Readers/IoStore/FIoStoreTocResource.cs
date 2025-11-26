using System.Text;
using UAssetEditor.Binary;
using UAssetEditor.Encryption.Aes;

namespace UAssetEditor.Unreal.Readers.IoStore;

// Might have taken a little from
// https://github.com/FabianFG/CUE4Parse/blob/master/CUE4Parse/UE4/IO/Objects/FIoStoreTocResource.cs
public class FIoStoreTocResource
{
    private readonly Reader Reader;
    
    public readonly FIoStoreTocHeader Header;

    public readonly FIoChunkId[] ChunkIds;
    public readonly FIoOffsetAndLength[] OffsetAndLengths;
    public readonly FIoStoreTocCompressedBlockEntry[] CompressionBlocks;
    public readonly int[]? ChunkPerfectHashSeeds;
    public readonly int[]? ChunkIndicesWithoutPerfectHash;
    public readonly string[] CompressionMethods;
    public readonly long DirectoryIndexPosition = -1;
    
    // Encryption
    private FAesKey? AesKey;
    public bool IsEncrypted => Header.ContainerFlags.HasFlag(EIoContainerFlags.Encrypted);

    public void SetAesKey(FAesKey key)
    {
        AesKey = key;
    }
    
    
    public FIoStoreTocResource(Reader reader)
    {
        Reader = reader;
        Header = new FIoStoreTocHeader(reader);
        
        ChunkIds = reader.ReadArray<FIoChunkId>((int)Header.TocEntryCount);

        OffsetAndLengths = new FIoOffsetAndLength[Header.TocEntryCount];
        for (int i = 0; i < OffsetAndLengths.Length; i++)
            OffsetAndLengths[i] = new FIoOffsetAndLength(reader);

        uint perfectHashSeedsCount = 0;
        uint chunksWithoutPerfectHashCount = 0;
        if (Header.Version > EIoStoreTocVersion.PerfectHashWithOverflow)
        {
            perfectHashSeedsCount = Header.TocChunkPerfectHashSeedsCount;
            chunksWithoutPerfectHashCount = Header.TocChunksWithoutPerfectHashCount;
        }
        else if (Header.Version >= EIoStoreTocVersion.PerfectHash)
        {
            perfectHashSeedsCount = Header.TocChunkPerfectHashSeedsCount;
        }
        if (perfectHashSeedsCount > 0)
        {
            ChunkPerfectHashSeeds = reader.ReadArray<int>((int)perfectHashSeedsCount);
        }
        if (chunksWithoutPerfectHashCount > 0)
        {
            ChunkIndicesWithoutPerfectHash = reader.ReadArray<int>((int)chunksWithoutPerfectHashCount);
        }

        CompressionBlocks = new FIoStoreTocCompressedBlockEntry[Header.TocCompressedBlockEntryCount];
        for (int i = 0; i < CompressionBlocks.Length; i++)
            CompressionBlocks[i] = new FIoStoreTocCompressedBlockEntry(reader);

        var length = (int)Header.CompressionMethodNameLength;
        
        CompressionMethods = new string[Header.CompressionMethodNameCount + 1];
        CompressionMethods[0] = "None";

        for (int i = 1; i < CompressionMethods.Length; i++)
        {
            var buffer = reader.ReadBytes(length);
            var str = Encoding.ASCII.GetString(buffer).TrimEnd('\0');

            CompressionMethods[i] = str;
        }

        if (Header.ContainerFlags.HasFlag(EIoContainerFlags.Signed))
        {
            var hashSize = reader.Read<int>();
            reader.Position += hashSize + hashSize + 20 * Header.TocCompressedBlockEntryCount; // 20 = sizeof(FSHAHash)
        }

        if (Header.Version >= EIoStoreTocVersion.DirectoryIndex
            && Header.ContainerFlags.HasFlag(EIoContainerFlags.Indexed)
            && Header.DirectoryIndexSize > 0)
        {
            DirectoryIndexPosition = reader.Position;
        }
    }

    public unsafe ulong HashChunkIdWithSeed(int seed, FIoChunkId chunk)
    {
        var ptr = (byte*)&chunk;
        var size = sizeof(FIoChunkId);
        var hash = seed != 0 ? (ulong)seed : 0xcbf29ce484222325;
       
        for (uint index = 0; index < size; index++)
        {
            hash = (hash * 0x00000100000001B3) ^ ptr[index];
        }

        return hash;
    }
}