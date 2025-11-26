using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties.Types;

public class UInt64Property : AbstractProperty<ulong>
{
    public UInt64Property()
    { }
    
    public UInt64Property(ulong value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Value = reader.Read<ulong>();
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        writer.Write(Value);
    }
}