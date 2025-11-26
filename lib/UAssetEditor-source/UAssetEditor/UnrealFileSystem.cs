using System.Collections.Concurrent;
using OodleDotNet;
using Serilog;
using UAssetEditor.Encryption.Aes;
using UAssetEditor.Unreal.Assets;
using UAssetEditor.Unreal.Containers;
using UAssetEditor.Unreal.Misc;
using UAssetEditor.Unreal.Packages;
using UAssetEditor.Unreal.Readers;
using UAssetEditor.Unreal.Readers.IoStore;
using UAssetEditor.Utils;
using UsmapDotNet;

namespace UAssetEditor;

public class UnrealFileSystem
{
    private List<ContainerFile> _containers;
    public IReadOnlyList<ContainerFile> Containers => _containers;
    public Dictionary<FGuid, FAesKey> AesKeys = new();
    
    private string _directory;
    public readonly IEnumerable<string> Files;
    public int MountedFiles { get; private set; }
    
    public ConcurrentDictionary<string, UnrealFileEntry> Packages { get; } = new();

    public Usmap? Mappings { get; private set; }

    public IoGlobalReader? GetGlobalReader() => Containers.FirstOrDefault(x => x.Reader is IoGlobalReader)?.Reader as IoGlobalReader;
    
    public UnrealFileSystem(string directory)
    {
        _directory = directory;
        Files = Directory.EnumerateFiles(_directory);
        _containers = new List<ContainerFile>();
    }

    public void LoadMappings(string path, string? oodleDll = null)
    {
        Mappings = new Usmap(path, new UsmapOptions
        {
            Oodle = string.IsNullOrEmpty(oodleDll) ? null : new Oodle(oodleDll),
            SaveNames = false
        });
    }
    
    public void LoadMappings(byte[] data, string? oodleDll = null)
    {
        Mappings = new Usmap(data, new UsmapOptions
        {
            Oodle = string.IsNullOrEmpty(oodleDll) ? null : new Oodle(oodleDll),
            SaveNames = false
        });
    }
    
    public void Initialize(bool loadInParallel = true, int maxDegreeOfParallelism = 6, bool unloadContainersWithNoFiles = true)
    {
        var files = Directory.EnumerateFiles(_directory);

        if (loadInParallel)
        {
            Parallel.ForEach(files, new ParallelOptions
                { MaxDegreeOfParallelism = maxDegreeOfParallelism }, x => HandleContainer(x, unloadContainersWithNoFiles));
        }
        else
        {
            foreach (var file in files)
                HandleContainer(file, unloadContainersWithNoFiles);
        }
        
        if (unloadContainersWithNoFiles)
            GC.Collect();
    }

    public void HandleContainer(string file, bool unloadContainerIfNoFilesFound)
    {
        Information($"Mounting '{file}'");

        ContainerFile? container = null;
        
        if (file.EndsWith(".utoc"))
        {
            if (file.EndsWith(".o.utoc")) // Skip Fortnite's UEFN containers
                return;

            if (file.EndsWith("global.utoc"))
            {
                container = new IoFile(IoGlobalReader.InitializeGlobalData(file), this);
            }
            else
            {
                container = new IoFile(file, this);
                container.Mount();
                MountedFiles++;

                if (unloadContainerIfNoFilesFound)
                {
                    if (container.FileCount == 0)
                    {
                        container.Unmount();
                        Information($"Unmounted '{file}' because no virtual files were found.");
                        return;
                    }
                }

                foreach (var pkg in container.PackagesByPath)
                    Packages.TryAdd(pkg.Key, pkg.Value);
            }

            _containers.Add(container);
        }
        else if (file.EndsWith(".pak"))
        {
            // TODO paks
        }
        
        Information($"Mounted '{file}' with {container?.FileCount ?? 0} files.");
    }

    public bool TryGetPackage(string packagePath, out UnrealFileEntry? pkg, out ContainerFile? file)
    {
        foreach (var container in Containers)
        {
            if (!container.TryFindPackage(packagePath.ToLower(), out pkg))
                continue;

            file = container;
            return true;
        }

        pkg = null;
        file = null;
        return false;
    }
    
    public bool TryRead(string packagePath, out byte[] data)
    {
        if (!TryGetPackage(packagePath, out var pkg, out _))
        {
            data = [];
            return false;
        }

        data = pkg!.Read();
        return true;
    }

    public bool TryExtractAsset(string packagePath, out Asset? asset)
    {
        if (!TryGetPackage(packagePath, out var pkg, out var ctn))
        {
            asset = null;
            return false;
        }

        switch (pkg)
        {
            case FIoStoreEntry ioEntry:
            {
                var data = ioEntry.Read();
                asset = new ZenAsset(data, this, ctn?.Reader as UnrealFileReader);

                var globalToc = GetGlobalReader();
                asset.As<ZenAsset>().Initialize(globalToc!);

                asset.Mappings = Mappings;

                return true;
            }
            case FPakEntry pakEntry:
                throw new NotImplementedException();
        }

        asset = null;
        return false;
    }
}