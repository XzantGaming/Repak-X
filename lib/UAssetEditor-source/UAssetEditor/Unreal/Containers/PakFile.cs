namespace UAssetEditor.Unreal.Containers;

public class PakFile : ContainerFile
{
    public PakFile(string path, UnrealFileSystem? system = null) : base(path, system)
    {
    }

    public override bool IsEncrypted { get; }

    public override void Mount()
    {
        throw new NotImplementedException();
    }

    public override void Unmount()
    {
        throw new NotImplementedException();
    }
}