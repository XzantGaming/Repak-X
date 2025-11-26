using UAssetEditor.Binary;

namespace UAssetEditor.Unreal.Readers.IoStore;

public class FIoOffsetAndLength
{
    public long Position;
    
    public ulong Offset;
    public ulong Length;
    
    public FIoOffsetAndLength(Reader reader)
    {
        Position = reader.Position;
        Offset = reader.ReadBytesInterpreted<ulong>(5, true);
        Length = reader.ReadBytesInterpreted<ulong>(5, true);
    }
}