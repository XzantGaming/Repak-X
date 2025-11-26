namespace UAssetEditor.Unreal.Names;

public struct FMappedName
{
    private const int IndexBits = 30;
    private const uint IndexMask = (1u << IndexBits) - 1u;
        
    private uint _nameIndex;
    public uint ExtraIndex;

    public FMappedName(uint nameIndex, uint extraIndex)
    {
        _nameIndex = nameIndex;
        ExtraIndex = extraIndex;
    }
        
    public uint NameIndex => _nameIndex & IndexMask;
}