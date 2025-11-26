using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using UAssetEditor.Classes;
using UAssetEditor.Unreal.Exports;

namespace UAssetEditor.Unreal.Properties.Structs.Misc;

public struct FFrameNumber : IUnrealType
{
    [UField]
    public int Value;
}
