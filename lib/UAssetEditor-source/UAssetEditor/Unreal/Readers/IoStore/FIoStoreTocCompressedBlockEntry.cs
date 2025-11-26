using UAssetEditor.Binary;

namespace UAssetEditor.Unreal.Readers.IoStore;

public class FIoStoreTocCompressedBlockEntry
{
    public long Position;
    
    public long Offset;
    public uint CompressedSize;
    public uint UncompressedSize;
    public byte CompressionMethodIndex;

    public bool IsCompressed() => CompressionMethodIndex != 0;

    public FIoStoreTocCompressedBlockEntry(Reader reader)
    {
        Position = reader.Position;

        Offset = reader.ReadBytesInterpreted<long>(5);
        CompressedSize = reader.ReadBytesInterpreted<uint>(3);
        UncompressedSize = reader.ReadBytesInterpreted<uint>(3);
        CompressionMethodIndex = reader.ReadByte();
    }

    public (int index, long offset) GetContainerIndexAndOffset(IoStoreReader reader)
    {
        var index = (int) ((ulong) Offset / reader.Resource.Header.PartitionSize);
        var offset =  (long) ((ulong) Offset % reader.Resource.Header.PartitionSize);

        return (index, offset);
    }
}