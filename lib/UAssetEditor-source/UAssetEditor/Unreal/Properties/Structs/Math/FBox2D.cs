using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Exports;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public class FBox2D : UStruct, IUnrealType 
{
    [UField]
    public FVector2D Min;
    [UField]
    public FVector2D Max;
    [UField]
    public byte bIsValid;

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Min = reader.Read<FVector2D>();
        Max = reader.Read<FVector2D>();
        bIsValid = reader.Read<byte>();
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(Min);
        writer.Write(Max);
        writer.WriteByte(bIsValid);
    }
}

