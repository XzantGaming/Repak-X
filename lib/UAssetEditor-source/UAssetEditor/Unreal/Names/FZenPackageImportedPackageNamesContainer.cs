using UAssetEditor.Binary;

namespace UAssetEditor.Unreal.Names;

// https://github.com/EpicGames/UnrealEngine/blob/585df42eb3a391efd295abd231333df20cddbcf3/Engine/Source/Runtime/CoreUObject/Private/Serialization/AsyncLoading2.cpp#L107
public class FZenPackageImportedPackageNamesContainer
{
    public NameMapContainer NameMap;
    public int[] Numbers;

    public FZenPackageImportedPackageNamesContainer(Reader reader)
    {
        NameMap = NameMapContainer.ReadNameMap(reader);
        Numbers = reader.ReadArray<int>(NameMap.Length);
    }

    public void Serialize(Writer writer)
    {
        NameMapContainer.WriteNameMap(writer, NameMap);
        writer.WriteArray(Numbers);
    }
}