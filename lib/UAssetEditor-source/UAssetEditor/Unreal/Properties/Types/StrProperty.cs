using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties.Types;

public class StrProperty : AbstractProperty<string>
{
    public StrProperty()
    { }
    
    public StrProperty(string value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Value = FString.Read(reader);
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        FString.Write(writer, Value ?? string.Empty);
    }
}