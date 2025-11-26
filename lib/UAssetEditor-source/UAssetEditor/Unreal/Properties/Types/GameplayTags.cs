using System.Data;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Types;

public class GameplayTagArrayProperty : AbstractProperty<List<FName>>
{
    public GameplayTagArrayProperty()
    { }
    
    public GameplayTagArrayProperty(List<FName> value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Value = [];

        if (mode == ESerializationMode.Zero)
            return;
        
        if (asset is null)
            throw new NoNullAllowedException("Asset cannot be null.");
        
        var tagCount = reader.Read<uint>();

        for (int i = 0; i < tagCount; i++)
            Value.Add(new FName(reader, asset.NameMap));
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (Value is null)
        {
            writer.Write(0);
            return;
        }
        
        if (asset is null)
            throw new NoNullAllowedException("Asset cannot be null.");
        
        writer.Write(Value.Count);
        
        foreach (var tag in Value)
        {
            tag.Serialize(writer, asset.NameMap);
        }
    }
}