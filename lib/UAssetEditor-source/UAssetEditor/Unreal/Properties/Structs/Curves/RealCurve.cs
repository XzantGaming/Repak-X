// https://github.com/FabianFG/CUE4Parse/blob/784b39afcec3d906f9830cc051c0d21378d02ebf/CUE4Parse/UE4/Objects/Engine/Curves/RealCurve.cs

using UAssetEditor.Classes;
using UAssetEditor.Unreal.Exports;

namespace UAssetEditor.Unreal.Properties.Structs.Curves;

/** Method of interpolation between this key and the next. */
public enum ERichCurveInterpMode : byte
{
    /** Use linear interpolation between values. */
    RCIM_Linear,
    /** Use a constant value. Represents stepped values. */
    RCIM_Constant,
    /** Cubic interpolation. See TangentMode for different cubic interpolation options. */
    RCIM_Cubic,
    /** No interpolation. */
    RCIM_None
}

/** Enumerates extrapolation options. */
public enum ERichCurveExtrapolation : byte
{
    /** Repeat the curve without an offset. */
    RCCE_Cycle,
    /** Repeat the curve with an offset relative to the first or last key's value. */
    RCCE_CycleWithOffset,
    /** Sinusoidally extrapolate. */
    RCCE_Oscillate,
    /** Use a linearly increasing value for extrapolation.*/
    RCCE_Linear,
    /** Use a constant value for extrapolation */
    RCCE_Constant,
    /** No Extrapolation */
    RCCE_None
}

/** A rich, editable float curve */
public abstract class FRealCurve : UStruct, IUnrealType
{
    public float DefaultValue;
    public ERichCurveExtrapolation PreInfinityExtrap;
    public ERichCurveExtrapolation PostInfinityExtrap;
    
    public FRealCurve()
    {
        DefaultValue = 3.402823466e+38f; // MAX_flt
        PreInfinityExtrap = ERichCurveExtrapolation.RCCE_Constant;
        PostInfinityExtrap = ERichCurveExtrapolation.RCCE_Constant;
    }
}