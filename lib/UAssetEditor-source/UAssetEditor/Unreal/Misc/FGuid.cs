using System.Globalization;
using System.Runtime.InteropServices;
using System.Text;
using UAssetEditor.Classes;

namespace UAssetEditor.Unreal.Misc;

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct FGuid : IUnrealType
{
    [UField] public uint A;
    [UField] public uint B;
    [UField] public uint C;
    [UField] public uint D;

    // https://github.com/FabianFG/CUE4Parse/blob/6afbbddaabd51bfb501db9c4edbcbc3ae276b853/CUE4Parse/UE4/Objects/Core/Misc/FGuid.cs#L52
    public FGuid(ReadOnlySpan<char> hexString)
    {
        A = uint.Parse(hexString.Slice(0, 8), NumberStyles.HexNumber);
        B = uint.Parse(hexString.Slice(8, 8), NumberStyles.HexNumber);
        C = uint.Parse(hexString.Slice(16, 8), NumberStyles.HexNumber);
        D = uint.Parse(hexString.Slice(24, 8), NumberStyles.HexNumber);
    }
    
    public override string ToString()
    {
        var sb = new StringBuilder();
        sb.Append(A.ToString("X8"));
        sb.Append(B.ToString("X8"));
        sb.Append(C.ToString("X8"));
        sb.Append(D.ToString("X8"));
        
        return sb.ToString();
    }   
}