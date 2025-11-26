namespace UAssetEditor.Unreal.Properties.Unversioned;

// Based heavily on https://github.com/FabianFG/CUE4Parse/blob/master/CUE4Parse/UE4/Assets/Objects/Unversioned/FFragment.cs
public struct FFragment
{
    public const uint SkipMax = 127;
    public const uint ValueMax = 127;

    private const uint SkipNumMask = 0x007fu;
    private const uint HasZeroMask = 0x0080u;
    private const int ValueNumShift = 9;
    private const uint IsLastMask  = 0x0100u;
        
    public byte SkipNum; // Number of properties to skip before values
    public bool bHasAnyZeroes;
    public byte ValueNum;  // Number of subsequent property values stored
    public bool bIsLast; // Is this the last fragment of the header?

    public FFragment(ushort packed)
    {
        SkipNum = (byte) (packed & SkipNumMask);
        bHasAnyZeroes = (packed & HasZeroMask) != 0;
        ValueNum = (byte) (packed >> ValueNumShift);
        bIsLast = (packed & IsLastMask) != 0;
    }

    public ushort Pack()
    {
        return (ushort)(SkipNum | (ushort)(bHasAnyZeroes ? HasZeroMask : 0) | (ushort)(ValueNum << ValueNumShift) |
                        (ushort)(bIsLast ? IsLastMask : 0));
    }
}