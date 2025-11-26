using System.Data;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties.Types;

public class NameProperty : AbstractProperty<FName>
{
    public NameProperty()
    { }
    
    public NameProperty(FName value)
    {
        Value = value;
    }
    
    public override string ToString()
    {
        return Value?.Name ?? "None";
    }

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        if (mode == ESerializationMode.Zero)
        {
            Value = new FName("None");
            return;
        }

        ArgumentNullException.ThrowIfNull(asset);

        var value = new FName(reader, asset.NameMap);
        Value = value;
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        if (Value is null)
            throw new NoNullAllowedException("FName cannot be null. Remove from properties.");
        
        ArgumentNullException.ThrowIfNull(asset);
        
        Value.Serialize(writer, asset.NameMap);
    }
}