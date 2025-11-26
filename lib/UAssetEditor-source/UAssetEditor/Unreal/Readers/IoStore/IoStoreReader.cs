using System.Data;
using Serilog;
using UAssetEditor.Binary;
using UAssetEditor.Compression;
using UAssetEditor.Encryption.Aes;
using UAssetEditor.Unreal.Containers;
using UAssetEditor.Unreal.Names;
using UAssetEditor.Unreal.Packages;

namespace UAssetEditor.Unreal.Readers.IoStore;

public class IoStoreReader : UnrealFileReader
{
    public FIoStoreTocResource Resource;
    public FIoContainerHeader ContainerHeader;
    public Dictionary<FIoChunkId, FIoOffsetAndLength>? TocImperfectHashMapFallback;

    private bool bHasPerfectHashMap => Resource.ChunkPerfectHashSeeds != null;
    
    public override bool IsEncrypted => Resource.IsEncrypted;
    public override string[] CompressionMethods => Resource.CompressionMethods;
    public bool HasDirectoryIndex => Resource.DirectoryIndexPosition > 0;

    private List<Reader> Archives = new();
    public Reader GetArchive(int partitionIndex) => Archives.ElementAt(partitionIndex);
    
    public IoStoreReader(ContainerFile? owner, string file) : base(owner, file)
    {
        var reader = new Reader(file);
        Resource = new FIoStoreTocResource(reader);

        for (int i = 0; i < Resource.Header.PartitionCount; i++)
        {
            var path = Name.Replace(".utoc", i > 0 ? $"_s{i}.ucas" : ".ucas");
            Archives.Add(new Reader(path));
        }

        // https://github.com/FabianFG/CUE4Parse/blob/7b085b0a374edf1e59f552b50751e9b2f3960499/CUE4Parse/UE4/IO/IoStoreReader.cs#L92
        if (Resource.ChunkPerfectHashSeeds != null)
        {
            TocImperfectHashMapFallback = new();
            if (Resource.ChunkIndicesWithoutPerfectHash != null)
            {
                foreach (var chunkIndexWithoutPerfectHash in Resource.ChunkIndicesWithoutPerfectHash)
                {
                    TocImperfectHashMapFallback[Resource.ChunkIds[chunkIndexWithoutPerfectHash]] =
                        Resource.OffsetAndLengths[chunkIndexWithoutPerfectHash];
                }
            }
        }
    }

    public FIoOffsetAndLength? FindChunkImperfect(FIoChunkId chunkId)
    {
        if (TocImperfectHashMapFallback != null)
        {
            return TocImperfectHashMapFallback.GetValueOrDefault(chunkId);
        }

        var chunkIndex = Array.IndexOf(Resource.ChunkIds, chunkId);
        return chunkIndex == -1 ? null : Resource.OffsetAndLengths[chunkIndex];
    }

    public FIoOffsetAndLength? FindChunkInternal(FIoChunkId chunkId)
    {
        if (bHasPerfectHashMap)
        {
            var chunkCount = (uint)Resource.ChunkIds.Length;
            if (chunkCount == 0)
                return null;

            var seedCount = (uint)Resource.ChunkPerfectHashSeeds!.Length;
            var seedIndex = (uint)(Resource.HashChunkIdWithSeed(0, chunkId) % seedCount);
            var seed = Resource.ChunkPerfectHashSeeds[seedIndex];
            if (seed == 0)
                return null;

            var slot = 0U;
            if (seed < 0)
            {
                var seedAsIndex = (uint)(-seed - 1);
                if (seedAsIndex < chunkCount)
                {
                    slot = seedAsIndex;
                }
                else
                {
                    return FindChunkImperfect(chunkId);
                }
            }
            else
            {
                slot = (uint)(Resource.HashChunkIdWithSeed(seed, chunkId) % chunkCount);
            }

            return Resource.ChunkIds[slot].GetHashCode() == chunkId.GetHashCode() ? Resource.OffsetAndLengths[slot] : null;
        }

        return null;
    }
    
    public byte[] ExtractChunk(FIoChunkId chunkId)
    {
        var offsetAndLength = FindChunkInternal(chunkId);
        if (offsetAndLength == null)
            throw new ApplicationException(
                $"Could not find chunk with id {chunkId.ChunkId} and type {chunkId.ChunkType}");
        
        var blocks = FIoStoreEntry.GetCompressionBlocks(this, offsetAndLength);

        var data = new byte[offsetAndLength.Length];
        ReadBlocks(blocks, data);
        
        return data;
    }

    public void ReadBlocks(FIoStoreTocCompressedBlockEntry[] blocks, byte[] data)
    {
        var offset = 0;
        
        foreach (var block in blocks)
        {
            var indexAndOffset = block.GetContainerIndexAndOffset(this);
            var archive = GetArchive(indexAndOffset.index);

            archive.Position = indexAndOffset.offset;
            var buffer = DecryptIfEncrypted(archive.ReadBytes((int)block.CompressedSize));
            var decompressed = CompressionHandler.HandleDecompression(this, block.CompressionMethodIndex, buffer, (int)block.UncompressedSize);

            if (offset + decompressed.Length > data.Length)
                throw new OutOfMemoryException("The buffer given is too small to receive all of these blocks");
            
            Buffer.BlockCopy(decompressed, 0, data, offset, decompressed.Length);
            offset += decompressed.Length;
        }
    }
    
    public override void ProcessIndex()
    {
        if (!HasDirectoryIndex || Resource.Header.DirectoryIndexSize == 0 || !IsAssignedAes)
            return;
        
        Position = Resource.DirectoryIndexPosition;

        var buffer = ReadBytes((int)Resource.Header.DirectoryIndexSize);
        var decrypted = DecryptIfEncrypted(buffer);
        var indexReader = new Reader(Name + " - Directory Index", decrypted);

        try
        {
            MountPoint = FString.Read(indexReader);
        }
        catch
        {
            throw new InvalidDataException("AES key is invalid, unable to decrypt.");
        }
        
        var directoryEntries = indexReader.ReadArray<FIoDirectoryIndexEntry>();
        var fileEntries = indexReader.ReadArray<FIoFileIndexEntry>();

        var stringTableSize = indexReader.Read<int>();
        var stringTable = indexReader.ReadArray(FString.Read, stringTableSize);

        _packagesByPath = new Dictionary<string, UnrealFileEntry>();
        ReadIndex(MountPoint, 0U);
        IsMounted = true;
        return;

        // https://github.com/FabianFG/CUE4Parse/blob/0b9616c806ba53c112cf2805ad3f9f823dbb35d6/CUE4Parse/UE4/IO/IoStoreReader.cs#L333
        void ReadIndex(string directoryName, uint dir)
        {
            const uint invalidHandle = uint.MaxValue;
            
            while (dir != invalidHandle)
            {
                ref var dirEntry = ref directoryEntries[dir];
                var subDirectoryName = dirEntry.Name == invalidHandle ? directoryName : $"{directoryName}{stringTable[dirEntry.Name]}/";

                var file = dirEntry.FirstFileEntry;
                while (file != invalidHandle)
                {
                    ref var fileEntry = ref fileEntries[file];

                    var path = string.Concat(subDirectoryName, stringTable[fileEntry.Name]);
                    var entry = new FIoStoreEntry(path, Owner, fileEntry.UserData);
                    
                    _packagesByPath[path.ToLower()] = entry;
                    file = fileEntry.NextFileEntry;
                }

                ReadIndex(subDirectoryName, dirEntry.FirstChildEntry);
                dir = dirEntry.NextSiblingEntry;
            }
        }
    }

    public override void Unmount()
    {
        Resource = null;
        ContainerHeader = null;
        TocImperfectHashMapFallback = null;
    }

    public void ReadContainerHeader()
    {
        var chunkId = new FIoChunkId(Resource.Header.ContainerId.Id, 0, EIoChunkType5.ContainerHeader);

        byte[]? data = null;
        
        try
        {
            if (IsAssignedAes)
                data = ExtractChunk(chunkId);
            else
            {
                Log.Error($"'{Owner?.Path}' ({Resource.Header.EncryptionKeyGuid}) has not been assigned an AesKey");
            }
        }
        catch (Exception ex)
        {
            Log.Error(ex.ToString());
        }

        if (data == null)
        {
            Log.Error("Unable to extract ContainerHeader chunk!");
            return;
        }

        var reader = new Reader(data);
        ContainerHeader = new FIoContainerHeader(reader);
    }
}