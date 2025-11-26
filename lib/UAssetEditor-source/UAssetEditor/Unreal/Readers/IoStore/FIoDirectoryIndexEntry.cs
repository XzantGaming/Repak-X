namespace UAssetEditor.Unreal.Readers.IoStore;

public struct FIoDirectoryIndexEntry
{
    public uint Name;
    public uint FirstChildEntry;
    public uint NextSiblingEntry;
    public uint FirstFileEntry;
}