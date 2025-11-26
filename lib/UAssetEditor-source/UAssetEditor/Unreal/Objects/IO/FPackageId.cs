using System.Diagnostics;
using System.Text;
using UAssetEditor.Unreal.Misc;

namespace UAssetEditor.Unreal.Objects.IO;

public struct FPackageId
{
    public ulong Id;

    public FPackageId(ulong id)
    {
        Id = id;
    }
    
    public static FPackageId FromName(string name)
    {
        var nameBuf = Encoding.Unicode.GetBytes(name.ToLower());
        var hash = CityHash.CityHash64(nameBuf);
        return new FPackageId(hash);
    }
}