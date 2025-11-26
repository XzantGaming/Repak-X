using Serilog;
using UAssetEditor.Binary;
using UAssetEditor.Unreal.Packages;
using UAssetEditor.Unreal.Readers;
using UAssetEditor.Unreal.Readers.IoStore;
using UAssetEditor.Utils;

namespace UAssetEditor.Unreal.Containers;

public abstract class ContainerFile : IDisposable
{
    public Reader? Reader { get; protected set; }
    public abstract bool IsEncrypted { get; }
    
    public string Path;
    public UnrealFileSystem? System;

    public ContainerFile(string path, UnrealFileSystem? system = null)
    {
        Path = path;
        System = system;
    }

    public IReadOnlyDictionary<string, UnrealFileEntry> PackagesByPath => 
        Reader?.AsOrDefault<UnrealFileReader>()?.PackagesByPath ?? new Dictionary<string, UnrealFileEntry>();

    public int FileCount => PackagesByPath.Count;
    
    /// <summary>
    /// Loads the container and all of its data
    /// </summary>
    public abstract void Mount();
    public abstract void Unmount();

    /// <summary>
    /// Attempts to find a package via its path.
    /// </summary>
    /// <param name="path"></param>
    /// <param name="pkg"></param>
    /// <returns></returns>
    public bool TryFindPackage(string path, out UnrealFileEntry? pkg)
    {
        var reader = Reader?.AsOrDefault<UnrealFileReader>();
        if (reader is null)
        {
            if (Reader is not IoGlobalReader)
                Log.Logger.Warning($"'{path}' was unable to be found in '{Reader?.Name}' because the reader is not an unreal file reader.");
            
            pkg = null;
            return false;
        }
        
        if (!string.IsNullOrEmpty(reader.MountPoint))
            path = path.StartsWith(reader.MountPoint) ? path : reader.MountPoint + path;
        
        return PackagesByPath.TryGetValue(path, out pkg);
    }

    public void Dispose()
    {
        Reader?.Dispose();
    }
}