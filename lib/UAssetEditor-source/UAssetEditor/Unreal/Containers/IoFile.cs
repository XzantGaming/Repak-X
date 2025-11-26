using UAssetEditor.Binary;
using UAssetEditor.Unreal.Objects.IO;
using UAssetEditor.Unreal.Packages;
using UAssetEditor.Unreal.Readers.IoStore;
using UAssetEditor.Utils;

namespace UAssetEditor.Unreal.Containers;

public class IoFile : ContainerFile
{
    public Dictionary<FPackageId, FIoStoreEntry>? FilesById;
    public IoStoreReader ReaderAsIoReader => Reader!.As<IoStoreReader>();
    public FIoStoreTocHeader Header => Resource.Header;
    public FIoStoreTocResource Resource => ReaderAsIoReader.Resource;
    public uint CompressionBlockSize => Header.CompressionBlockSize;
    
    public override bool IsEncrypted => Resource.IsEncrypted;
    
    public IoFile(string path, UnrealFileSystem? system = null) : base(path, system)
    {
        Reader = new IoStoreReader(this, path);
        
        if (System?.AesKeys.TryGetValue(Header.EncryptionKeyGuid, out var key) ?? false)
            ReaderAsIoReader.SetAesKey(key);
    }
    
    public IoFile(Reader reader, UnrealFileSystem? system = null) : base("", system)
    {
        Reader = reader;
    }

    public override void Mount()
    {
        if (Reader is not IoStoreReader)
            throw new ApplicationException("Cannot mount a non I/O store container");
        
        ReaderAsIoReader.ProcessIndex();
        FilesById = new Dictionary<FPackageId, FIoStoreEntry>();

        foreach (var pkg in PackagesByPath)
        {
            var entry = (FIoStoreEntry)pkg.Value;
            FilesById[entry.GetChunkId().AsPackageId()] = entry;
        }

        ReaderAsIoReader.ReadContainerHeader();
    }

    public override void Unmount()
    {
        FilesById = null;
        ReaderAsIoReader.Unmount();
        ReaderAsIoReader.Dispose();
    }
}