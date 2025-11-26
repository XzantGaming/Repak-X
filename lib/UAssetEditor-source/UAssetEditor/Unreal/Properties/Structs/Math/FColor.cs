using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UAssetEditor.Classes;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FColor : IUnrealType
{
    [UField]
    public byte B;
    [UField]
    public byte G;
    [UField]
    public byte R;
    [UField]
    public byte A;
}
