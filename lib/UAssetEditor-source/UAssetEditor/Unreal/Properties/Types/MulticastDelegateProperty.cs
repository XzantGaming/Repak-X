using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Objects.UObject;

namespace UAssetEditor.Unreal.Properties.Types;

public class MulticastDelegateProperty : AbstractProperty<FMulticastScriptDelegate>
{
    public MulticastDelegateProperty()
    { }
    
    public MulticastDelegateProperty(FMulticastScriptDelegate value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        if (asset == null)
            throw new NullReferenceException("Asset cannot be null");
        
        Value = mode == ESerializationMode.Zero
            ? new FMulticastScriptDelegate([])
            : new FMulticastScriptDelegate(asset);
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        writer.Write(Value);
    }
}