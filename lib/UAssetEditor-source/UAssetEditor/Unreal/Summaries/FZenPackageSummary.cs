using System.Runtime.CompilerServices;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Binary;
using UAssetEditor.Summaries;
using UAssetEditor.Unreal.Assets;

namespace UAssetEditor.Unreal.Summaries;

public struct FZenPackageSummary
{
    public static int Size => Unsafe.SizeOf<FZenPackageSummary>();
    
    public uint bHasVersioningInfo;
    public uint HeaderSize;
    public FMappedName Name;
    public EPackageFlags PackageFlags;
    public uint CookedHeaderSize;
    public int ImportedPublicExportHashesOffset;
    public int ImportMapOffset;
    public int ExportMapOffset;
    public int ExportBundleEntriesOffset;
    public int DependencyBundleHeadersOffset;
    public int DependencyBundleEntriesOffset;
    public int ImportedPackageNamesOffset;
}