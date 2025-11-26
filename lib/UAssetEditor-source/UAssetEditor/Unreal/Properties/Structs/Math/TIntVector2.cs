using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Exports;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public class TIntVector2<T> : UStruct, IUnrealType
{
    [UField]
    public T? X;
    
    [UField]
    public T? Y;

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        if (mode == ESerializationMode.Zero)
        {
            return;
        }

        X = reader.Read<T>();
        Y = reader.Read<T>();
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(X);
        writer.Write(Y);
    }
}