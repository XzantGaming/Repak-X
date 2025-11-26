using UAssetEditor.Unreal.Exports;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FQuat : IUnrealType
{
    [UField]
    public float X;
    
    [UField]
    public float Y;
    
    [UField]
    public float Z;
    
    [UField]
    public float W;
}

public struct FQuat_LARGE_WORLD_COORDINATES : IUnrealType
{
    [UField]
    public double X;
    
    [UField]
    public double Y;
    
    [UField]
    public double Z;
    
    [UField]
    public double W;
}