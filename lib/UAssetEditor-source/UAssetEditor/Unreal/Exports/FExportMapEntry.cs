using UAssetEditor.Unreal.Names;
using UAssetEditor.Summaries;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Objects;

namespace UAssetEditor.Unreal.Exports;

public class FExportMapEntry
{
    public const int Size = 72;

    public ulong CookedSerialOffset;
    public ulong CookedSerialSize;
    public FMappedName ObjectName;
    public FPackageObjectIndex OuterIndex;
    public FPackageObjectIndex ClassIndex;
    public FPackageObjectIndex SuperIndex;
    public FPackageObjectIndex TemplateIndex;
    public ulong PublicExportHash;
    public EObjectFlags ObjectFlags;
    public byte FilterFlags; // EExportFilterFlags: client/server flags

    public ZenAsset Asset;
    public string Name;
    public Lazy<string> Class;
    
    public FExportMapEntry(ZenAsset reader)
    {
        Asset = reader;
        CookedSerialOffset = reader.Read<ulong>();
        CookedSerialSize = reader.Read<ulong>();
        ObjectName = reader.Read<FMappedName>();
        OuterIndex = reader.Read<FPackageObjectIndex>();
        ClassIndex = reader.Read<FPackageObjectIndex>();
        SuperIndex = reader.Read<FPackageObjectIndex>();
        TemplateIndex = reader.Read<FPackageObjectIndex>();
        PublicExportHash = reader.Read<ulong>();
        ObjectFlags = reader.Read<EObjectFlags>();
        FilterFlags = reader.Read<byte>();
        reader.Position += 3;
        
        Name = reader.NameMap[(int)ObjectName.NameIndex];
        Class = new Lazy<string>(() => Asset.ResolveObjectIndex(ClassIndex)?.Name.ToString() ?? "None");
    }

    public void Serialize(Writer writer)
    {
        writer.Write(CookedSerialOffset);
        writer.Write(CookedSerialSize);
        writer.Write(ObjectName);
        writer.Write(OuterIndex);
        writer.Write(ClassIndex);
        writer.Write(SuperIndex);
        writer.Write(TemplateIndex);
        writer.Write(PublicExportHash);
        writer.Write(ObjectFlags);
        writer.WriteByte(FilterFlags);
        writer.Position += 3; // Padding
    }
    
    public void SetObjectName(string name)
    {
        var i = Asset.ReferenceOrAddString(name);
        ObjectName = new FMappedName((uint)i, 0);
    }
}