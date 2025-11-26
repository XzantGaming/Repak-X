using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UAssetEditor.Classes;

namespace UAssetEditor.Unreal.Properties.Structs.Math;

public struct FIntVector : IUnrealType
{
    [UField]
    public int X;
    [UField]
    public int Y;
    [UField]
    public int Z;
}
