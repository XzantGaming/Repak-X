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

public class FBox : UStruct, IUnrealType
{
    public FVector3 Min;
    public FVector3 Max;
    public byte IsValid;

    public override void Read(Reader reader, PropertyData? data, Asset? asset = null,
        ESerializationMode mode = ESerializationMode.Normal)
    {
        Min = reader.Read<FVector3>();
        Max = reader.Read<FVector3>();
        IsValid = reader.Read<byte>();
    }

    public override void Write(Writer writer, Asset? asset = null)
    {
        writer.Write(Min);
        writer.Write(Max);
        writer.WriteByte(IsValid);
    }
}
