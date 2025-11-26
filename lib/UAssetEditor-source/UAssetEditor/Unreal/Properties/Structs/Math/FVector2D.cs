using UAssetEditor.Unreal.Exports;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FVector2D : IUnrealType
{
    [UField]
    public float X;
    
    [UField]
    public float Y;
}

public struct FVector2D_LARGE_WORLD_COORDINATES : IUnrealType
{
    [UField]
    public double X;
    
    [UField]
    public double Y;
}