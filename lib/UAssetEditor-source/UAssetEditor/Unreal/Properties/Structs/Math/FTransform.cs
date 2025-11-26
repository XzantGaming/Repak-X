using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UAssetEditor.Classes;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FTransform : IUnrealType
{
    [UField]
    public FQuat Rotation;
    [UField]
    public FVector3 Translation;
    [UField]
    public FVector3 Scale3D;
}
