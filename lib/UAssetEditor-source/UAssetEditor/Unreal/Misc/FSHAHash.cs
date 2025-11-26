using UAssetEditor.Binary;

namespace UAssetEditor.Unreal.Misc;

public class FSHAHash
{
    public const int SIZE = 20;

    public readonly byte[] Hash;
    
    public FSHAHash(Reader reader)
    {
        Hash = reader.ReadBytes(SIZE);
    }
}