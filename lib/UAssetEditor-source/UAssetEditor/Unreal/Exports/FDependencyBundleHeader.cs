namespace UAssetEditor.Unreal.Exports;

public struct FDependencyBundleHeader
{
    public int FirstEntryIndex;
    public uint[][] EntryCount; // 2 * 2
}