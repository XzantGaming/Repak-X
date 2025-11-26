using UAssetEditor.Unreal.Misc;
using UAssetEditor.Unreal.Versioning;
using UAssetEditor.Binary;

namespace UAssetEditor.Summaries;

public class FPackageFileSummary
{
    public const uint PACKAGE_FILE_TAG = 0x9E2A83C1U;
    public const uint PACKAGE_FILE_TAG_SWAPPED = 0xC1832A9EU;
    public const uint PACKAGE_FILE_TAG_ACE7 = 0x37454341U; // ACE7
    private const uint PACKAGE_FILE_TAG_ONE = 0x00656E6FU; // SOD2

    public readonly uint Tag;
    public readonly int LegacyFileVersion;
    public readonly int LegacyUE3Version;
    public readonly int LegacyUE4Version;
    public readonly int LegacyUE5Version;
    public readonly EUnrealEngineObjectLicenseeUEVersion FileVersionLicenseeUE;
    public EPackageFlags PackageFlags;
    public int TotalHeaderSize;
    public readonly string FolderName;
    public int NameCount;
    public readonly int NameOffset;
    public readonly int SoftObjectPathsCount;
    public readonly int SoftObjectPathsOffset;
    public readonly string? LocalizationId;
    public readonly int GatherableTextDataCount;
    public readonly int GatherableTextDataOffset;
    public int ExportCount;
    public readonly int ExportOffset;
    public int ImportCount;
    public readonly int ImportOffset;
    public readonly int DependsOffset;
    public readonly int SoftPackageReferencesCount;
    public readonly int SoftPackageReferencesOffset;
    public readonly int SearchableNamesOffset;
    public readonly int ThumbnailTableOffset;
    public readonly FGuid Guid;
    public readonly FGuid PersistentGuid;
    // TODO
    /*public readonly FGenerationInfo[] Generations;
    public readonly FEngineVersion? SavedByEngineVersion;
    public readonly FEngineVersion? CompatibleWithEngineVersion;
    public readonly ECompressionFlags CompressionFlags;*/
    public readonly int PackageSource;
    public bool bUnversioned;
    public readonly int AssetRegistryDataOffset;
    public int BulkDataStartOffset; // serialized as long
    public readonly int WorldTileInfoDataOffset;
    public readonly int[] ChunkIds;
    public readonly int PreloadDependencyCount;
    public readonly int PreloadDependencyOffset;
    public readonly int NamesReferencedFromExportDataCount;
    public readonly long PayloadTocOffset;
    public readonly int DataResourceOffset;

    public FPackageFileSummary(Reader r)
    {
        Tag = r.Read<uint>();
        LegacyFileVersion = r.Read<int>();

        if (LegacyFileVersion < 0)
        {
            if (LegacyFileVersion != 4)
                LegacyUE3Version = r.Read<int>();

            LegacyUE4Version = r.Read<int>();

            if (LegacyFileVersion <= -8)
                LegacyUE5Version = r.Read<int>();

            FileVersionLicenseeUE = r.Read<EUnrealEngineObjectLicenseeUEVersion>();
            // TODO version container
        }

        TotalHeaderSize = r.Read<int>();
        FolderName = r.ReadString();
        PackageFlags = r.Read<EPackageFlags>();
        NameCount = r.Read<int>();
        NameOffset = r.Read<int>();
        
        // TODO everything else
    }
}