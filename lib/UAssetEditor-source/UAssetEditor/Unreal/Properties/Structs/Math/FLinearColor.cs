using UAssetEditor.Unreal.Exports;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FLinearColor : IUnrealType
{
    [UField]
    public float R;
    
    [UField]
    public float G;
    
    [UField]
    public float B;
    
    [UField]
    public float A;
}