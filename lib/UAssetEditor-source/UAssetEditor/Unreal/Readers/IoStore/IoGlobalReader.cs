using UAssetEditor.Binary;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Unreal.Objects;

namespace UAssetEditor.Unreal.Readers.IoStore;

public readonly struct FScriptObjectEntry
{
    public readonly FMappedName ObjectName;
    public readonly FPackageObjectIndex GlobalIndex;
    public readonly FPackageObjectIndex OuterIndex;
    public readonly FPackageObjectIndex CDOClassIndex;

    public FName GetObjectName(IoGlobalReader globalData) => new(ObjectName, globalData.GlobalNameMap);
}

public class IoGlobalReader : Reader
{
    public NameMapContainer GlobalNameMap;
    public Dictionary<FPackageObjectIndex, FScriptObjectEntry> ScriptObjectEntriesMap = new();
 
    public string GetScriptName(FPackageObjectIndex index) => GlobalNameMap[ScriptObjectEntriesMap[index].ObjectName.NameIndex];
    
    public IoGlobalReader(byte[] data) : base(data)
    { }

    public static IoGlobalReader InitializeGlobalData(string path)
    {
        var ioStoreReader = new IoStoreReader(null, path);
        var data = ioStoreReader.ExtractChunk(new FIoChunkId(0, 0, EIoChunkType5.ScriptObjects));
        var reader = new IoGlobalReader(data);
        
        reader.GlobalNameMap = NameMapContainer.ReadNameMap(reader);

        var numScriptObjects = reader.Read<int>();
        var scriptObjectEntries = reader.ReadArray<FScriptObjectEntry>(numScriptObjects);

        foreach (var obj in scriptObjectEntries)
            reader.ScriptObjectEntriesMap[obj.GlobalIndex] = obj;

        return reader;
    }
}