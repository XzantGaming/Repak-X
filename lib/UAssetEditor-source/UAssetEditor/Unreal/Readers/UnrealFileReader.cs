using System.Data;
using UAssetEditor.Binary;
using UAssetEditor.Encryption.Aes;
using UAssetEditor.Unreal.Containers;
using UAssetEditor.Unreal.Packages;

namespace UAssetEditor.Unreal.Readers;

public abstract class UnrealFileReader : Reader
{
    internal Dictionary<string, UnrealFileEntry>? _packagesByPath { get; set; }
    public IReadOnlyDictionary<string, UnrealFileEntry> PackagesByPath => _packagesByPath ?? new();

    public ContainerFile? Owner { get; }

    protected UnrealFileReader(ContainerFile? owner, string file) : base(file)
    {
        Owner = owner;
    }
    
    public abstract bool IsEncrypted { get; }
    public abstract string[] CompressionMethods { get; }
    public bool IsMounted { get; protected set; }
    public string? MountPoint { get; protected set; }

    public FAesKey? AesKey;
    
    /// <summary>
    /// Does this container have an aes key assigned to it?
    /// </summary>
    public bool IsAssignedAes => AesKey != null; 

    public void SetAesKey(FAesKey key)
    {
        AesKey = key;
    }

    public abstract void ProcessIndex();
    public abstract void Unmount();

    public byte[] DecryptIfEncrypted(byte[] buffer)
    {
        if (!IsEncrypted) 
            return buffer;
        
        if (!IsAssignedAes)
            throw new NoNullAllowedException($"{nameof(AesKey)} cannot be null when attempting to decrypt.");
            
        return buffer.Decrypt(AesKey!);
    }
}