using UAssetEditor.Unreal.Exports;
using UAssetEditor.Unreal.Objects;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Properties;

namespace UAssetEditor.Unreal.Assets;

public class PakAsset : Asset
{
    public PakAsset(byte[] data) : base(data)
    {
    }

    public PakAsset(string path) : base(path)
    {
    }

    public override void ReadAll()
    {
        throw new NotImplementedException();
    }

    public override uint ReadHeader()
    {
        throw new NotImplementedException();
    }

    public override List<UProperty> ReadProperties(UStruct structure)
    {
        throw new NotImplementedException();
    }

    public override void WriteProperties(Writer writer, string type, List<UProperty> properties)
    {
        throw new NotImplementedException();
    }

    public override void WriteAll(Writer writer)
    {
        throw new NotImplementedException();
    }

    public override void WriteHeader(Writer writer)
    {
        throw new NotImplementedException();
    }

    public override ResolvedObject? ResolvePackageIndex(FPackageIndex? index)
    {
        throw new NotImplementedException();
    }
}