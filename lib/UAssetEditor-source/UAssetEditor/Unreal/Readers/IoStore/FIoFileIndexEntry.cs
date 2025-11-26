namespace UAssetEditor.Unreal.Readers.IoStore;

public struct FIoFileIndexEntry
{
    public uint Name;
    public uint NextFileEntry;
    public uint UserData;
}