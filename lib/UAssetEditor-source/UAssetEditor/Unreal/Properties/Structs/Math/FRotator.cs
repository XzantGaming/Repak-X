using UAssetEditor.Unreal.Exports;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FRotator : IUnrealType
{
    [UField]
    public float Pitch;
    
    [UField]
    public float Yaw;
    
    [UField]
    public float Roll;
}

public struct FRotator_LARGE_WORLD_COORDINATES : IUnrealType
{
    [UField]
    public double Pitch;
    
    [UField]
    public double Yaw;
    
    [UField]
    public double Roll;
}