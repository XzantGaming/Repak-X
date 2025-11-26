using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using UAssetEditor.Unreal.Objects.IO;

namespace UAssetEditor.Unreal.Readers.IoStore;

public enum EIoChunkType5 : byte
{
    Invalid = 0,
    ExportBundleData = 1,
    BulkData = 2,
    OptionalBulkData = 3,
    MemoryMappedBulkData = 4,
    ScriptObjects = 5,
    ContainerHeader = 6,
    ExternalFile = 7,
    ShaderCodeLibrary = 8,
    ShaderCode = 9,
    PackageStoreEntry = 10,
    DerivedData = 11,
    EditorDerivedData = 12
}

[StructLayout(LayoutKind.Sequential, Pack = 1)]
public record struct FIoChunkId
{
    public ulong ChunkId;
    public ushort ChunkIndex;
    private byte _padding;
    public EIoChunkType5 ChunkType;

    public FIoChunkId(ulong id, ushort index, EIoChunkType5 type)
    {
        ChunkId = id;
        ChunkIndex = index;
        ChunkType = type;
    }

    public FPackageId AsPackageId() => new(ChunkId);
}