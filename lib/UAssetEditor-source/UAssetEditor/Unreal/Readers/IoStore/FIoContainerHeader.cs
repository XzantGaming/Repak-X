using Serilog;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Unreal.Objects.IO;

namespace UAssetEditor.Unreal.Readers.IoStore;

public enum EIoContainerHeaderVersion
{
    BeforeVersionWasAdded = -1, // Custom constant to indicate pre-UE5 data
    Initial = 0,
    LocalizedPackages = 1,
    OptionalSegmentPackages = 2,
    NoExportInfo = 3,

    LatestPlusOne,
    Latest = LatestPlusOne - 1
}


public class FIoContainerHeader
{
    private const int Signature = 0x496f436e;
    
    public EIoContainerHeaderVersion Version;
    public FIoContainerId ContainerId;
    public FPackageId[] PackageIds;
    public FFilePackageStoreEntry[] StoreEntries; //FPackageStoreEntry[PackageIds.Num()]
    public FPackageId[] OptionalSegmentPackageIds;
    public FFilePackageStoreEntry[] OptionalSegmentStoreEntries; //FPackageStoreEntry[OptionalSegmentPackageIds.Num()]
	public NameMapContainer RedirectsNameMap;
	/*public FIoContainerHeaderLocalizedPackage[] LocalizedPackages;
	public FIoContainerHeaderPackageRedirect[] PackageRedirects;
	public FIoContainerHeaderSoftPackageReferences[] SoftPackageReferences;*/
    
    public FIoContainerHeader(Reader reader)
    {
        var signature = reader.Read<int>();
        if (signature != Signature)
        {
            Log.Error($"Invalid container header signature 0x{signature:X8} != 0x{Signature:X8} --- Aes Key is most likely invalid.");
            //throw new ApplicationException($"Invalid container header signature 0x{signature:X8} != 0x{Signature:X8}");
        }

        Version = reader.Read<EIoContainerHeaderVersion>();

        if (Version < EIoContainerHeaderVersion.Latest)
            throw new NotImplementedException("Reading earlier IO Store versions is not implemented yet.");

        ContainerId = reader.Read<FIoContainerId>();
        PackageIds = reader.ReadArray<FPackageId>();
        var storeEntriesSize = reader.Read<int>();
        var storeEntriesEnd = reader.Position + storeEntriesSize;
        StoreEntries = reader.ReadArray(() => new FFilePackageStoreEntry(reader, Version), PackageIds.Length);
        reader.Position = storeEntriesEnd;
        OptionalSegmentPackageIds = reader.ReadArray<FPackageId>();
        OptionalSegmentStoreEntries = reader.ReadArray(() => new FFilePackageStoreEntry(reader, Version), OptionalSegmentPackageIds.Length);
        RedirectsNameMap = NameMapContainer.ReadNameMap(reader);
    }
}