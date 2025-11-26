using UAssetEditor.Unreal.Exports;
using UAssetEditor.Binary;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FIntPoint : IUnrealType
{
    [UField]
    public float X;
    
    [UField]
    public float Y;
}