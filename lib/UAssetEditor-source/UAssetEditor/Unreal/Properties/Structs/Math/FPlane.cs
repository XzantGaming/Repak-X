using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UAssetEditor.Classes;

namespace UAssetEditor.Unreal.Properties.Structs.Math;


public struct FPlane : IUnrealType
{
    [UField]
    public FVector3 Vector;
    [UField]
    public float W;
}
public struct FPlane_LARGE_WORLD_COORDINATES : IUnrealType
{
    [UField]
    public FVector3_LARGE_WORLD_COORDINATES Vector;
    [UField]
    public double W;
}
