using System.Runtime.InteropServices;
using UAssetEditor.Classes;

namespace UAssetEditor.Unreal.Properties.Structs.Curves;

// https://github.com/FabianFG/CUE4Parse/blob/master/CUE4Parse/UE4/Objects/Engine/Curves/SimpleCurve.cs
public class SimpleCurve
{
    [StructLayout(LayoutKind.Sequential)]
    public struct FSimpleCurveKey : IUnrealType
    {
        public float Time;
        public float Value;
    }

    public class FSimpleCurve : FRealCurve
    {
        public ERichCurveInterpMode InterpMode;
        public FSimpleCurveKey[] Keys;
    }
}