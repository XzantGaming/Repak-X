using UAssetEditor.Binary;
using UAssetEditor.Unreal.Misc;
using UAssetEditor.Unreal.Objects.IO;

namespace UAssetEditor.Unreal.Readers.IoStore;

public class FFilePackageStoreEntry
{
    public int ExportCount;
    public int ExportBundleCount;
    public FPackageId[] ImportedPackages;
    public FSHAHash[] ShaderMapHashes;

    public FFilePackageStoreEntry(Reader reader, EIoContainerHeaderVersion version)
    {
        if (version >= EIoContainerHeaderVersion.Initial)
        {
            if (version < EIoContainerHeaderVersion.NoExportInfo)
            {
                ExportCount = reader.Read<int>();
                ExportBundleCount = reader.Read<int>();
            }

            ImportedPackages = reader.ReadCArrayView<FPackageId>();
            ShaderMapHashes = reader.ReadCArrayView(() => new FSHAHash(reader));
        }
        else
        {
            reader.Position += 8; // ExportBundlesSize
            ExportCount = reader.Read<int>();
            ExportBundleCount = reader.Read<int>();
            reader.Position += 8; // LoadOrder + Pad
            ImportedPackages = reader.ReadCArrayView<FPackageId>();
        }
    }
}