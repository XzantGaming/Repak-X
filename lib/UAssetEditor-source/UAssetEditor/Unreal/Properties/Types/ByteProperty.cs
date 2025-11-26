using UAssetEditor.Binary;
using UAssetEditor.Unreal.Assets;


namespace UAssetEditor.Unreal.Properties.Types;

public class ByteProperty : AbstractProperty<byte>
{
    public ByteProperty()
    { }

    public ByteProperty(byte value)
    {
        Value = value;
    }
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Value = mode == ESerializationMode.Zero ? (byte)0 : reader.ReadByte();
    }

    public override void Write(Writer writer, UProperty property, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        writer.WriteByte(Value);
    }
}