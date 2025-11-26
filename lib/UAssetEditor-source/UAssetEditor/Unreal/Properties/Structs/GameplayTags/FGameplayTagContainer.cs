using System.Data;
using UAssetEditor.Unreal.Exports;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.GameplayTags;

public class FGameplayTagContainer : UStruct, IUnrealType
{
    [UField]
    public List<FName>? Tags;

    public static List<FName> ReadGameplayTagArray(Asset? asset, Reader reader)
    {
        if (asset is null)
            throw new NoNullAllowedException("Asset cannot be null.");
        
        var tags = new List<FName>();
        var tagCount = reader.Read<uint>();

        for (int i = 0; i < tagCount; i++)
            tags.Add(new FName(reader, asset.NameMap));

        return tags;
    }
    
    public static void WriteGameplayTagArray(Writer writer, List<FName> names, Asset asset)
    {
        if (asset is null)
            throw new NoNullAllowedException("Asset cannot be null.");
        
        writer.Write(names.Count);

        foreach (var name in names)
            name.Serialize(writer, asset.NameMap);
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Tags = ReadGameplayTagArray(asset, reader);
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        ArgumentNullException.ThrowIfNull(asset);
        WriteGameplayTagArray(writer, Tags ?? throw new NoNullAllowedException($"{nameof(Tags)} cannot be null."), asset);
    }
}