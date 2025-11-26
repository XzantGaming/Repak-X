using UAssetEditor.Binary;
using UAssetEditor.Unreal.Names;

namespace UAssetEditor.Unreal.Objects;

public class FCellExportMapEntry
{
    public ulong CookedSerialOffset;
    public ulong CookedSerialLayoutSize;
    public ulong CookedSerialSize;
    public FMappedName CppClassInfo;
    public ulong PublicExportHash;

    public FCellExportMapEntry(Reader reader)
    {
        CookedSerialOffset = reader.Read<ulong>();
        CookedSerialLayoutSize = reader.Read<ulong>();
        CookedSerialSize = reader.Read<ulong>();
        CppClassInfo = reader.Read<FMappedName>();
        PublicExportHash = reader.Read<ulong>();
    }

    public void Serialize(Writer writer)
    {
        writer.Write(CookedSerialOffset);
        writer.Write(CookedSerialLayoutSize);
        writer.Write(CookedSerialSize);
        writer.Write(CppClassInfo);
        writer.Write(PublicExportHash);
    }
}