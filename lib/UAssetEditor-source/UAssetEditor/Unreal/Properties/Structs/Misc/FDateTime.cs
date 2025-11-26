using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Exports;

namespace UAssetEditor.Unreal.Properties.Structs.Misc;

public class FDateTime : UStruct, IUnrealType
{
    [UField]
    public long Ticks;
    
    public override void Read(Reader reader, PropertyData? data, Asset? asset = null, ESerializationMode mode = ESerializationMode.Normal)
    {
        Ticks = reader.Read<long>();
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(Ticks);
    }

    public override string ToString()
    {
        return Ticks.ToString();
    }
}