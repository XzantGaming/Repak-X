

using UAssetEditor.Unreal.Objects;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Types;

public class ObjectProperty : AbstractProperty<FPackageIndex>
{
    public ObjectProperty()
    { }
    
    public ObjectProperty(FPackageIndex value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Value = new FPackageIndex(asset, mode == ESerializationMode.Zero ? 0 : reader.Read<int>());
    }
    
    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        writer.Write(Value?.Index ?? 0);
    }
}