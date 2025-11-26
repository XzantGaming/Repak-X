using UAssetEditor.Unreal.Exports;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FVector3 : IUnrealType
{
    [UField] public float A;
    [UField] public float B;
    [UField] public float C;
}

public struct FVector3_LARGE_WORLD_COORDINATES : IUnrealType
{
    [UField] public double A;
    [UField] public double B;
    [UField] public double C;
}