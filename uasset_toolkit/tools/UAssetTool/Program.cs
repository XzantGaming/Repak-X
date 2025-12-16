#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using UAssetAPI;
using UAssetAPI.UnrealTypes;
using UAssetAPI.ExportTypes;
using UAssetAPI.ExportTypes.Texture;
using UAssetAPI.Unversioned;
using UAssetAPI.PropertyTypes.Objects;

namespace UAssetTool;

/// <summary>
/// Unified UAsset Tool - Combines detection, fixing, and patching for all UE asset types.
/// Supports both interactive JSON mode (stdin/stdout) and CLI mode.
/// </summary>
public class Program
{
    public static async Task<int> Main(string[] args)
    {
        // CLI mode: command-line arguments
        if (args.Length > 0)
        {
            return RunCliMode(args);
        }
        
        // Interactive JSON mode: read from stdin
        return await RunInteractiveMode();
    }

    #region CLI Mode
    
    private static int RunCliMode(string[] args)
    {
        string command = args[0].ToLower();
        
        try
        {
            return command switch
            {
                "detect" => CliDetect(args),
                "fix" => CliFix(args),
                "batch_detect" => CliBatchDetect(args),
                "dump" => CliDump(args),
                "help" or "--help" or "-h" => CliHelp(),
                _ => throw new Exception($"Unknown command: {command}")
            };
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"Error: {ex.Message}");
            if (Environment.GetEnvironmentVariable("DEBUG") == "1")
            {
                Console.Error.WriteLine(ex.StackTrace);
            }
            return 1;
        }
    }
    
    private static int CliHelp()
    {
        Console.WriteLine("UAssetTool - Unified UE Asset Tool");
        Console.WriteLine();
        Console.WriteLine("Usage: UAssetTool <command> [args]");
        Console.WriteLine();
        Console.WriteLine("Commands:");
        Console.WriteLine("  detect <uasset_path> [usmap_path]       - Detect asset type");
        Console.WriteLine("  fix <uasset_path> [usmap_path]          - Fix SerializeSize for meshes");
        Console.WriteLine("  batch_detect <directory> [usmap_path]   - Detect all assets in directory");
        Console.WriteLine("  dump <uasset_path> <usmap_path>         - Dump detailed asset info");
        Console.WriteLine();
        Console.WriteLine("Interactive mode: Run without arguments to use JSON stdin/stdout");
        return 0;
    }
    
    private static int CliDetect(string[] args)
    {
        if (args.Length < 2)
        {
            Console.Error.WriteLine("Usage: UAssetTool detect <uasset_path> [usmap_path]");
            return 1;
        }

        string uassetPath = args[1];
        string? usmapPath = args.Length > 2 ? args[2] : null;

        if (!File.Exists(uassetPath))
        {
            Console.Error.WriteLine($"File not found: {uassetPath}");
            return 1;
        }

        var asset = LoadAsset(uassetPath, usmapPath);
        var assetType = DetectAssetType(asset);
        
        var result = new
        {
            path = uassetPath,
            asset_type = assetType,
            export_count = asset.Exports.Count,
            import_count = asset.Imports.Count
        };

        Console.WriteLine(JsonSerializer.Serialize(result, new JsonSerializerOptions { WriteIndented = true }));
        return 0;
    }
    
    private static int CliFix(string[] args)
    {
        if (args.Length < 2)
        {
            Console.Error.WriteLine("Usage: UAssetTool fix <uasset_path> [usmap_path]");
            return 1;
        }

        string uassetPath = args[1];
        string? usmapPath = args.Length > 2 ? args[2] : null;

        if (!File.Exists(uassetPath))
        {
            Console.Error.WriteLine($"File not found: {uassetPath}");
            return 1;
        }

        var result = FixSerializeSize(uassetPath, usmapPath);
        Console.WriteLine(JsonSerializer.Serialize(result, new JsonSerializerOptions { WriteIndented = true }));
        return 0;
    }
    
    private static int CliBatchDetect(string[] args)
    {
        if (args.Length < 2)
        {
            Console.Error.WriteLine("Usage: UAssetTool batch_detect <directory> [usmap_path]");
            return 1;
        }

        string directory = args[1];
        string? usmapPath = args.Length > 2 ? args[2] : null;

        if (!Directory.Exists(directory))
        {
            Console.Error.WriteLine($"Directory not found: {directory}");
            return 1;
        }

        var results = new List<object>();
        var uassetFiles = Directory.GetFiles(directory, "*.uasset", SearchOption.AllDirectories);

        Console.Error.WriteLine($"Scanning {uassetFiles.Length} .uasset files...");

        Usmap? mappings = LoadMappings(usmapPath);

        foreach (var uassetPath in uassetFiles)
        {
            try
            {
                var asset = LoadAssetWithMappings(uassetPath, mappings);
                string assetType = DetectAssetType(asset);

                results.Add(new
                {
                    path = uassetPath,
                    asset_type = assetType,
                    file_name = Path.GetFileName(uassetPath)
                });
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Failed to process {uassetPath}: {ex.Message}");
            }
        }

        var grouped = results.GroupBy(r => ((dynamic)r).asset_type)
                            .Select(g => new
                            {
                                asset_type = g.Key,
                                count = g.Count(),
                                files = g.ToList()
                            })
                            .ToList();

        Console.WriteLine(JsonSerializer.Serialize(new
        {
            total_files = uassetFiles.Length,
            by_type = grouped
        }, new JsonSerializerOptions { WriteIndented = true }));

        return 0;
    }
    
    private static int CliDump(string[] args)
    {
        if (args.Length < 3)
        {
            Console.Error.WriteLine("Usage: UAssetTool dump <uasset_path> <usmap_path>");
            return 1;
        }

        string uassetPath = args[1];
        string usmapPath = args[2];

        if (!File.Exists(uassetPath))
        {
            Console.Error.WriteLine($"File not found: {uassetPath}");
            return 1;
        }

        var asset = LoadAsset(uassetPath, usmapPath);
        DumpAssetInfo(asset, uassetPath);
        return 0;
    }
    
    #endregion

    #region Interactive JSON Mode
    
    private static async Task<int> RunInteractiveMode()
    {
        try
        {
            string? line;
            while ((line = await Console.In.ReadLineAsync()) != null)
            {
                if (string.IsNullOrWhiteSpace(line)) continue;

                try 
                {
                    var request = JsonSerializer.Deserialize<UAssetRequest>(line);
                    if (request == null)
                    {
                        WriteJsonResponse(false, "Invalid JSON request");
                        continue;
                    }

                    var response = ProcessRequest(request);
                    var responseJson = JsonSerializer.Serialize(response);
                    Console.WriteLine(responseJson.Replace("\r", "").Replace("\n", ""));
                }
                catch (JsonException)
                {
                    WriteJsonResponse(false, "Invalid JSON format");
                }
            }
        }
        catch (Exception ex)
        {
            WriteJsonResponse(false, $"Unhandled exception: {ex.Message}");
        }
        
        return 0;
    }
    
    private static UAssetResponse ProcessRequest(UAssetRequest request)
    {
        try
        {
            return request.Action switch
            {
                // Single file detection - all use unified DetectAssetType
                "detect_texture" => DetectSingleAsset(request.FilePath, "texture"),
                "detect_mesh" => DetectSingleAsset(request.FilePath, "skeletal_mesh"),
                "detect_skeletal_mesh" => DetectSingleAsset(request.FilePath, "skeletal_mesh"),
                "detect_static_mesh" => DetectSingleAsset(request.FilePath, "static_mesh"),
                "detect_blueprint" => DetectSingleAsset(request.FilePath, "blueprint"),
                
                // Batch detection - all use unified workflow
                "batch_detect_skeletal_mesh" => BatchDetectAssetType(request.FilePaths, "skeletal_mesh"),
                "batch_detect_static_mesh" => BatchDetectAssetType(request.FilePaths, "static_mesh"),
                "batch_detect_texture" => BatchDetectAssetType(request.FilePaths, "texture"),
                "batch_detect_blueprint" => BatchDetectAssetType(request.FilePaths, "blueprint"),
                
                // Texture operations
                "set_mip_gen" => SetMipGen(request.FilePath, request.MipGen),
                "get_texture_info" => GetTextureInfo(request.FilePath),
                "convert_texture" => ConvertTexture(request.FilePath),
                "strip_mipmaps" => StripMipmaps(request.FilePath),
                "strip_mipmaps_native" => StripMipmapsNative(request.FilePath),
                "batch_strip_mipmaps_native" => BatchStripMipmapsNative(request.FilePaths),
                
                // Mesh operations
                "patch_mesh" => PatchMesh(request.FilePath, request.UexpPath),
                "get_mesh_info" => GetMeshInfo(request.FilePath),
                "fix_serialize_size" => FixSerializeSizeJson(request.FilePath, request.UsmapPath),
                
                // Debug
                "debug_asset_info" => DebugAssetInfo(request.FilePath),
                
                _ => new UAssetResponse { Success = false, Message = $"Unknown action: {request.Action}" }
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    #endregion

    #region Unified Asset Detection
    
    /// <summary>
    /// Core asset type detection - single unified method for all asset types.
    /// Returns: "static_mesh", "skeletal_mesh", "texture", "material_instance", "blueprint", "other"
    /// </summary>
    private static string DetectAssetType(UAsset asset)
    {
        foreach (var export in asset.Exports)
        {
            string className = GetExportClassName(asset, export);
            
            // Check class name against known types
            if (className.Equals("StaticMesh", StringComparison.OrdinalIgnoreCase))
                return "static_mesh";
            if (className.Equals("SkeletalMesh", StringComparison.OrdinalIgnoreCase))
                return "skeletal_mesh";
            if (className.Equals("Texture2D", StringComparison.OrdinalIgnoreCase))
                return "texture";
            if (className.Equals("MaterialInstanceConstant", StringComparison.OrdinalIgnoreCase) ||
                className.Equals("MaterialInstance", StringComparison.OrdinalIgnoreCase))
                return "material_instance";
            if (className.Contains("Blueprint", StringComparison.OrdinalIgnoreCase))
                return "blueprint";
            
            // Check export type name (fallback)
            string exportTypeName = export.GetType().Name;
            if (exportTypeName.Contains("StaticMesh", StringComparison.OrdinalIgnoreCase))
                return "static_mesh";
            if (exportTypeName.Contains("SkeletalMesh", StringComparison.OrdinalIgnoreCase))
                return "skeletal_mesh";
            if (exportTypeName.Contains("Texture2D", StringComparison.OrdinalIgnoreCase))
                return "texture";
        }
        
        // Filename heuristics as last resort
        string? fileName = asset.FilePath != null ? Path.GetFileNameWithoutExtension(asset.FilePath) : null;
        if (!string.IsNullOrEmpty(fileName))
        {
            if (fileName.StartsWith("SM_", StringComparison.OrdinalIgnoreCase))
                return "static_mesh";
            if (fileName.StartsWith("SK_", StringComparison.OrdinalIgnoreCase))
                return "skeletal_mesh";
            if (fileName.StartsWith("T_", StringComparison.OrdinalIgnoreCase))
                return "texture";
            if (fileName.StartsWith("MI_", StringComparison.OrdinalIgnoreCase))
                return "material_instance";
            if (fileName.StartsWith("BP_", StringComparison.OrdinalIgnoreCase))
                return "blueprint";
        }
        
        return "other";
    }
    
    /// <summary>
    /// Get the class name for an export (from import reference)
    /// </summary>
    private static string GetExportClassName(UAsset asset, Export export)
    {
        if (export.ClassIndex.IsImport())
        {
            var import = export.ClassIndex.ToImport(asset);
            if (import != null)
            {
                return import.ObjectName?.Value?.Value ?? "Unknown";
            }
        }
        return "Unknown";
    }
    
    /// <summary>
    /// Check if asset matches a specific type
    /// </summary>
    private static bool IsAssetType(UAsset asset, string targetType)
    {
        string detectedType = DetectAssetType(asset);
        return detectedType.Equals(targetType, StringComparison.OrdinalIgnoreCase);
    }
    
    /// <summary>
    /// Detect single asset and check if it matches target type
    /// </summary>
    private static UAssetResponse DetectSingleAsset(string? filePath, string targetType)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            var asset = LoadAsset(filePath, usmapPath);
            
            // For textures, also check if it needs MipGen fix
            if (targetType == "texture")
            {
                bool isTexture = IsAssetType(asset, "texture");
                bool needsFix = isTexture && IsTextureNeedingMipGenFix(asset);
                return new UAssetResponse
                {
                    Success = true,
                    Message = needsFix ? "Texture needs MipGen fix" : (isTexture ? "Texture already has NoMipmaps" : "Not a texture"),
                    Data = needsFix
                };
            }
            
            bool isMatch = IsAssetType(asset, targetType);
            return new UAssetResponse
            {
                Success = true,
                Message = isMatch ? $"File is {targetType}" : $"File is not {targetType}",
                Data = isMatch
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    /// <summary>
    /// Batch detect - check multiple files for a specific asset type
    /// </summary>
    private static UAssetResponse BatchDetectAssetType(List<string>? filePaths, string targetType)
    {
        if (filePaths == null || filePaths.Count == 0)
            return new UAssetResponse { Success = false, Message = "file_paths required" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            Usmap? mappings = LoadMappings(usmapPath);

            bool foundMatch = filePaths.AsParallel().Any(filePath =>
            {
                if (!File.Exists(filePath)) return false;
                try
                {
                    var asset = LoadAssetWithMappings(filePath, mappings);
                    
                    // For textures, check if it needs MipGen fix
                    if (targetType == "texture")
                    {
                        return IsAssetType(asset, "texture") && IsTextureNeedingMipGenFix(asset);
                    }
                    
                    return IsAssetType(asset, targetType);
                }
                catch
                {
                    return false;
                }
            });

            return new UAssetResponse
            {
                Success = true,
                Message = foundMatch ? $"Found {targetType} in batch" : $"No {targetType} found",
                Data = foundMatch
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Batch detection error: {ex.Message}" };
        }
    }
    
    #endregion

    #region Texture Operations
    
    private static bool IsTextureNeedingMipGenFix(UAsset asset)
    {
        foreach (var export in asset.Exports)
        {
            if (GetExportClassName(asset, export) == "Texture2D" && export is NormalExport normalExport)
            {
                foreach (var property in normalExport.Data)
                {
                    if (property.Name?.Value?.Value == "MipGenSettings")
                    {
                        if (property is EnumPropertyData enumProp)
                        {
                            string value = enumProp.Value?.Value?.Value ?? "";
                            return !value.Equals("TMGS_NoMipmaps", StringComparison.OrdinalIgnoreCase);
                        }
                        else if (property is BytePropertyData byteProp)
                        {
                            return byteProp.Value != 13; // 13 = TMGS_NoMipmaps
                        }
                    }
                }
                // MipGenSettings not found = using default (FromTextureGroup) = needs fix
                return true;
            }
        }
        return false;
    }
    
    private static UAssetResponse SetMipGen(string? filePath, string? mipGen)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (string.IsNullOrEmpty(mipGen))
            return new UAssetResponse { Success = false, Message = "MipGen setting required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            var asset = LoadAsset(filePath, usmapPath);
            asset.UseSeparateBulkDataFiles = true;
            
            if (!IsAssetType(asset, "texture"))
                return new UAssetResponse { Success = false, Message = "Asset is not a Texture2D" };
            
            byte mipGenValue = mipGen.ToLower() switch
            {
                "nomipmaps" => 13,
                "simpleaverage" => 1,
                "fromtexturegroup" => 0,
                _ => throw new Exception($"Unknown MipGen setting: {mipGen}")
            };
            
            bool patched = PatchTextureMipGenSettings(asset, mipGenValue);
            if (!patched)
                return new UAssetResponse { Success = false, Message = "Could not find MipGenSettings to patch" };
            
            File.Copy(filePath, filePath + ".backup", true);
            asset.Write(filePath);
            
            return new UAssetResponse { Success = true, Message = $"Set MipGenSettings to {mipGen}" };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    private static bool PatchTextureMipGenSettings(UAsset asset, byte mipGenValue)
    {
        bool patched = false;
        bool needsNewProperty = false;
        
        // First pass: check if we need to add a new property
        foreach (var export in asset.Exports)
        {
            if (GetExportClassName(asset, export) == "Texture2D" && export is NormalExport normalExport)
            {
                bool found = normalExport.Data.Any(p => p.Name?.Value?.Value == "MipGenSettings");
                if (!found) needsNewProperty = true;
                break;
            }
        }
        
        // Pre-add required names if needed
        if (needsNewProperty)
        {
            asset.AddNameReference(new FString("MipGenSettings"));
            asset.AddNameReference(new FString("TextureMipGenSettings"));
            asset.AddNameReference(new FString("EnumProperty"));
            asset.AddNameReference(new FString(GetMipGenSettingName(mipGenValue)));
        }
        
        // Second pass: patch
        foreach (var export in asset.Exports)
        {
            if (GetExportClassName(asset, export) == "Texture2D" && export is NormalExport normalExport)
            {
                var prop = normalExport.Data.FirstOrDefault(p => p.Name?.Value?.Value == "MipGenSettings");
                
                if (prop is EnumPropertyData enumProp)
                {
                    enumProp.Value = new FName(asset, GetMipGenSettingName(mipGenValue));
                    patched = true;
                }
                else if (prop is BytePropertyData byteProp)
                {
                    byteProp.Value = mipGenValue;
                    patched = true;
                }
                else if (prop == null)
                {
                    var newProp = new EnumPropertyData(new FName(asset, "MipGenSettings"));
                    newProp.EnumType = new FName(asset, "TextureMipGenSettings");
                    newProp.Value = new FName(asset, GetMipGenSettingName(mipGenValue));
                    normalExport.Data.Add(newProp);
                    patched = true;
                }
            }
        }
        
        return patched;
    }
    
    private static string GetMipGenSettingName(byte value) => value switch
    {
        0 => "TMGS_FromTextureGroup",
        1 => "TMGS_SimpleAverage",
        13 => "TMGS_NoMipmaps",
        _ => $"TMGS_Unknown_{value}"
    };
    
    /// <summary>
    /// Convert texture using UE4-DDS-Tools remove_mipmaps mode.
    /// This removes all mipmaps and converts the texture to inline format.
    /// </summary>
    private static UAssetResponse ConvertTexture(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            // Find UE4-DDS-Tools
            string? ddsToolsPath = FindUE4DDSTools();
            if (ddsToolsPath == null)
            {
                Console.Error.WriteLine("[UAssetTool] ERROR: UE4-DDS-Tools not found!");
                return new UAssetResponse { Success = false, Message = "UE4-DDS-Tools not found - check installation" };
            }
            
            string pythonExe = Path.Combine(ddsToolsPath, "python", "python.exe");
            string mainPy = Path.Combine(ddsToolsPath, "src", "main.py");
            
            Console.Error.WriteLine($"[UAssetTool] Python: {pythonExe}");
            Console.Error.WriteLine($"[UAssetTool] main.py: {mainPy}");
            
            if (!File.Exists(pythonExe))
            {
                Console.Error.WriteLine($"[UAssetTool] ERROR: Python not found at {pythonExe}");
                return new UAssetResponse { Success = false, Message = $"Python not found: {pythonExe}" };
            }
            if (!File.Exists(mainPy))
            {
                Console.Error.WriteLine($"[UAssetTool] ERROR: main.py not found at {mainPy}");
                return new UAssetResponse { Success = false, Message = $"main.py not found: {mainPy}" };
            }
            
            // Use remove_mipmaps mode - this removes mipmaps and converts to inline format
            Console.Error.WriteLine($"[UAssetTool] Running remove_mipmaps on: {filePath}");
            
            // Create a temp folder for output, then copy back
            string tempDir = Path.Combine(Path.GetTempPath(), "UAssetTool_RemoveMips_" + Guid.NewGuid().ToString("N")[..8]);
            Directory.CreateDirectory(tempDir);
            Console.Error.WriteLine($"[UAssetTool] Temp output folder: {tempDir}");
            
            try
            {
                var result = RunUE4DDSTools(pythonExe, mainPy,
                    $"\"{filePath}\" --mode=remove_mipmaps --version=5.3 --save_folder=\"{tempDir}\"");
                
                if (!result.Success)
                {
                    Console.Error.WriteLine($"[UAssetTool] remove_mipmaps failed: {result.Message}");
                    return new UAssetResponse { Success = false, Message = $"remove_mipmaps failed: {result.Message}" };
                }
                
                Console.Error.WriteLine($"[UAssetTool] remove_mipmaps output: {result.Message}");
                
                // Find and copy the output files back to original location
                string fileName = Path.GetFileNameWithoutExtension(filePath);
                string[] outputFiles = Directory.GetFiles(tempDir, $"{fileName}.*");
                
                Console.Error.WriteLine($"[UAssetTool] Found {outputFiles.Length} output files in temp folder");
                
                if (outputFiles.Length == 0)
                {
                    return new UAssetResponse { Success = false, Message = "No output files generated by remove_mipmaps" };
                }
                
                string? originalDir = Path.GetDirectoryName(filePath);
                foreach (var outputFile in outputFiles)
                {
                    string destFile = Path.Combine(originalDir ?? ".", Path.GetFileName(outputFile));
                    Console.Error.WriteLine($"[UAssetTool] Copying {outputFile} -> {destFile}");
                    File.Copy(outputFile, destFile, overwrite: true);
                }
                
                // Delete the .ubulk if it exists (data is now inline)
                string ubulkPath = Path.ChangeExtension(filePath, ".ubulk");
                if (File.Exists(ubulkPath))
                {
                    Console.Error.WriteLine($"[UAssetTool] Deleting .ubulk: {ubulkPath}");
                    File.Delete(ubulkPath);
                }
                
                Console.Error.WriteLine($"[UAssetTool] remove_mipmaps succeeded!");
                return new UAssetResponse 
                { 
                    Success = true, 
                    Message = "Texture mipmaps removed and converted to inline",
                    Data = new { output = result.Message, files_copied = outputFiles.Length }
                };
            }
            finally
            {
                // Cleanup temp directory
                try { Directory.Delete(tempDir, true); } catch { /* ignore */ }
            }
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[UAssetTool] Exception: {ex.Message}");
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    /// <summary>
    /// Strip mipmaps using UE4-DDS-Tools remove_mipmaps mode.
    /// </summary>
    private static UAssetResponse StripMipmaps(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? ddsToolsPath = FindUE4DDSTools();
            if (ddsToolsPath == null)
                return new UAssetResponse { Success = false, Message = "UE4-DDS-Tools not found" };
            
            string pythonExe = Path.Combine(ddsToolsPath, "python", "python.exe");
            string mainPy = Path.Combine(ddsToolsPath, "src", "main.py");
            
            Console.Error.WriteLine($"[UAssetTool] Removing mipmaps...");
            var result = RunUE4DDSTools(pythonExe, mainPy,
                $"\"{filePath}\" --mode=remove_mipmaps --version=5.3");
            
            return result;
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    /// <summary>
    /// Strip mipmaps using native UAssetAPI TextureExport.
    /// This is a pure C# implementation that doesn't require Python.
    /// </summary>
    private static UAssetResponse StripMipmapsNative(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            Console.Error.WriteLine($"[UAssetTool] Native mipmap stripping: {filePath}");
            
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            var asset = LoadAsset(filePath, usmapPath);
            
            // Find TextureExport
            TextureExport? textureExport = null;
            foreach (var export in asset.Exports)
            {
                if (export is TextureExport tex)
                {
                    textureExport = tex;
                    break;
                }
            }
            
            if (textureExport == null)
            {
                return new UAssetResponse { Success = false, Message = "No TextureExport found in asset" };
            }
            
            if (textureExport.PlatformData == null)
            {
                return new UAssetResponse { Success = false, Message = "TextureExport has no PlatformData (texture data not parsed)" };
            }
            
            int originalMipCount = textureExport.MipCount;
            Console.Error.WriteLine($"[UAssetTool] Original mip count: {originalMipCount}");
            
            if (originalMipCount <= 1)
            {
                return new UAssetResponse { Success = true, Message = "Texture already has 1 or fewer mipmaps" };
            }
            
            // The target data_resource_id should always be 5 for Marvel Rivals textures
            // All reference NoMipMaps textures use data_resource_id = 5 regardless of original structure
            int targetDataResourceId = 5;
            
            // Strip mipmaps
            bool stripped = textureExport.StripMipmaps();
            if (!stripped)
            {
                return new UAssetResponse { Success = false, Message = "Failed to strip mipmaps" };
            }
            
            Console.Error.WriteLine($"[UAssetTool] Stripped to {textureExport.MipCount} mipmap(s)");
            
            // Update DataResources - Match Python tool behavior:
            // Python outputs data_resource_id = max(original DataResourceIndex values)
            // with only 1 DataResource entry in .uasset
            // The game uses data_resource_id as a key that must match between .uexp and .uasset
            
            if (asset.DataResources != null && textureExport.PlatformData?.Mips?.Count > 0)
            {
                var mip = textureExport.PlatformData.Mips[0];
                int dataSize = mip.BulkData?.Data?.Length ?? 0;
                
                // Use targetDataResourceId calculated earlier (validMipCount or maxDataResourceIndex+1)
                
                // Create a new DataResource entry for the inline mip
                var inlineResource = new UAssetAPI.UnrealTypes.FObjectDataResource(
                    (UAssetAPI.UnrealTypes.EObjectDataResourceFlags)0,
                    0,  // SerialOffset - placeholder, will be updated
                    -1, // DuplicateSerialOffset
                    dataSize, // SerialSize
                    dataSize, // RawSize
                    new UAssetAPI.UnrealTypes.FPackageIndex(1), // OuterIndex
                    0x48 // LegacyBulkDataFlags - ForceInlinePayload | SingleUse
                );
                
                // Clear and add only 1 entry (matching Python's output structure)
                // But set the mip's DataResourceIndex to the original last index
                asset.DataResources.Clear();
                asset.DataResources.Add(inlineResource);
                
                // CRITICAL: Set the mip's DataResourceIndex to match Python's output
                // Python writes data_resource_id = original count - 1 (e.g., 5 for 6 entries)
                mip.BulkData.Header.DataResourceIndex = targetDataResourceId;
                
            }
            
            // Save the modified asset (first pass)
            asset.Write(filePath);
            
            // Second pass: Find the inline data offset in .uexp and update DataResource
            string uexpPath = Path.ChangeExtension(filePath, ".uexp");
            if (File.Exists(uexpPath) && asset.DataResources != null && asset.DataResources.Count > 0)
            {
                var mip = textureExport.PlatformData?.Mips?[0];
                if (mip?.BulkData?.Data != null && mip.BulkData.Data.Length >= 4)
                {
                    // Get the DataResource index we're using
                    int drIndex = mip.BulkData.Header.DataResourceIndex;
                    if (drIndex < 0 || drIndex >= asset.DataResources.Count)
                    {
                        drIndex = asset.DataResources.Count - 1;
                    }
                    
                    // Find the inline data by searching for the first 4 bytes of texture data
                    byte[] uexpData = File.ReadAllBytes(uexpPath);
                    byte[] searchPattern = new byte[4];
                    Array.Copy(mip.BulkData.Data, 0, searchPattern, 0, 4);
                    
                    long inlineOffset = -1;
                    for (int i = 0; i < uexpData.Length - 4; i++)
                    {
                        if (uexpData[i] == searchPattern[0] && 
                            uexpData[i+1] == searchPattern[1] &&
                            uexpData[i+2] == searchPattern[2] &&
                            uexpData[i+3] == searchPattern[3])
                        {
                            inlineOffset = i;
                            break;
                        }
                    }
                    
                    if (inlineOffset >= 0)
                    {
                        Console.Error.WriteLine($"[UAssetTool] Found inline data at offset {inlineOffset} (0x{inlineOffset:X})");
                        
                        // Update the DataResource SerialOffset at the correct index
                        var dr = asset.DataResources[drIndex];
                        asset.DataResources[drIndex] = new UAssetAPI.UnrealTypes.FObjectDataResource(
                            dr.Flags,
                            inlineOffset,  // Updated SerialOffset
                            dr.DuplicateSerialOffset,
                            dr.SerialSize,
                            dr.RawSize,
                            dr.OuterIndex,
                            dr.LegacyBulkDataFlags,
                            dr.CookedIndex
                        );
                        
                        // Write again with correct offset
                        asset.Write(filePath);
                        Console.Error.WriteLine($"[UAssetTool] Updated DataResource[{drIndex}] SerialOffset to {inlineOffset}");
                    }
                    else
                    {
                        Console.Error.WriteLine($"[UAssetTool] Warning: Could not find inline data offset");
                    }
                }
            }
            
            // CRITICAL FIX: Patch the data_resource_id in .uexp
            // The UAssetAPI write puts our value at the wrong position (64 bytes later than expected)
            // We need to find the inline data start and work backwards to find the data_resource_id
            if (File.Exists(uexpPath))
            {
                byte[] uexpBytes = File.ReadAllBytes(uexpPath);
                
                // The inline data starts at a known offset (found earlier as inlineOffset)
                // The data_resource_id is 4 bytes before the inline data
                // But we need to find where the FIRST data_resource_id is (the one that should have our target value)
                
                // For UE5.3+ textures, the structure before inline data is:
                // [mip_count=1][data_resource_id][inline_offset][data_size][width][height][depth]...[inline_data]
                // The data_resource_id we need to fix is right after mip_count
                
                // Find the position of mip_count=1 followed by data_resource_id=0
                // This is the position that should have our target value
                int targetValue = targetDataResourceId;
                
                // Only apply patch if targetValue is reasonable (< 100)
                if (targetValue > 0 && targetValue < 100)
                {
                    // Search for the specific pattern: [01 00 00 00][00 00 00 00] (mip_count=1, data_resource_id=0)
                    // followed later by [01 00 00 00][targetValue as int32] (mip_count=1, data_resource_id=targetValue)
                    int firstPos = -1;
                    int secondPos = -1;
                    
                    for (int i = 100; i < Math.Min(uexpBytes.Length - 8, 300); i++)
                    {
                        int val1 = BitConverter.ToInt32(uexpBytes, i);
                        int val2 = BitConverter.ToInt32(uexpBytes, i + 4);
                        
                        if (val1 == 1 && val2 == 0 && firstPos == -1)
                        {
                            firstPos = i + 4; // Position of data_resource_id (currently 0)
                        }
                        else if (val1 == 1 && val2 == targetValue && secondPos == -1 && firstPos >= 0)
                        {
                            secondPos = i + 4; // Position of data_resource_id (currently targetValue)
                        }
                    }
                    
                    // Swap values if both positions found and they're 64 bytes apart (expected offset)
                    if (firstPos >= 0 && secondPos >= 0 && (secondPos - firstPos) == 64)
                    {
                        byte[] targetBytes = BitConverter.GetBytes(targetValue);
                        byte[] zeroBytes = BitConverter.GetBytes(0);
                        
                        Array.Copy(targetBytes, 0, uexpBytes, firstPos, 4);
                        Array.Copy(zeroBytes, 0, uexpBytes, secondPos, 4);
                        
                        File.WriteAllBytes(uexpPath, uexpBytes);
                        Console.Error.WriteLine($"[UAssetTool] Patched data_resource_id: pos {firstPos}=0->{targetValue}, pos {secondPos}={targetValue}->0");
                    }
                }
            }
            
            // Delete .ubulk file if it exists (data is now inline)
            string ubulkPath = Path.ChangeExtension(filePath, ".ubulk");
            if (File.Exists(ubulkPath))
            {
                Console.Error.WriteLine($"[UAssetTool] Deleting .ubulk: {ubulkPath}");
                File.Delete(ubulkPath);
            }
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = $"Stripped mipmaps: {originalMipCount} -> {textureExport.MipCount}",
                Data = new { original_mips = originalMipCount, new_mips = textureExport.MipCount }
            };
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[UAssetTool] Native strip error: {ex.Message}");
            Console.Error.WriteLine($"[UAssetTool] Stack: {ex.StackTrace}");
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    /// <summary>
    /// Batch strip mipmaps from multiple textures using native UAssetAPI TextureExport.
    /// This processes all files in a single call for better performance.
    /// </summary>
    private static UAssetResponse BatchStripMipmapsNative(List<string>? filePaths)
    {
        if (filePaths == null || filePaths.Count == 0)
            return new UAssetResponse { Success = false, Message = "file_paths required" };

        try
        {
            Console.Error.WriteLine($"[UAssetTool] Batch stripping mipmaps for {filePaths.Count} files");
            
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            Usmap? mappings = LoadMappings(usmapPath);
            
            var results = new List<object>();
            int successCount = 0;
            int skipCount = 0;
            int errorCount = 0;
            
            foreach (var filePath in filePaths)
            {
                if (string.IsNullOrEmpty(filePath) || !File.Exists(filePath))
                {
                    results.Add(new { path = filePath, success = false, message = "File not found" });
                    errorCount++;
                    continue;
                }
                
                try
                {
                    var asset = LoadAssetWithMappings(filePath, mappings);
                    
                    // Find TextureExport
                    TextureExport? textureExport = null;
                    foreach (var export in asset.Exports)
                    {
                        if (export is TextureExport tex)
                        {
                            textureExport = tex;
                            break;
                        }
                    }
                    
                    if (textureExport == null)
                    {
                        results.Add(new { path = filePath, success = false, message = "No TextureExport found" });
                        errorCount++;
                        continue;
                    }
                    
                    if (textureExport.PlatformData == null)
                    {
                        results.Add(new { path = filePath, success = false, message = "No PlatformData (texture not parsed)" });
                        errorCount++;
                        continue;
                    }
                    
                    int originalMipCount = textureExport.MipCount;
                    
                    if (originalMipCount <= 1)
                    {
                        results.Add(new { path = filePath, success = true, message = "Already has 1 mipmap", skipped = true });
                        skipCount++;
                        continue;
                    }
                    
                    // Target data_resource_id = 5 for Marvel Rivals textures
                    int targetDataResourceId = 5;
                    
                    // Strip mipmaps
                    bool stripped = textureExport.StripMipmaps();
                    if (!stripped)
                    {
                        results.Add(new { path = filePath, success = false, message = "Failed to strip mipmaps" });
                        errorCount++;
                        continue;
                    }
                    
                    // Update DataResources
                    if (asset.DataResources != null && textureExport.PlatformData?.Mips?.Count > 0)
                    {
                        var mip = textureExport.PlatformData.Mips[0];
                        int dataSize = mip.BulkData?.Data?.Length ?? 0;
                        
                        var inlineResource = new UAssetAPI.UnrealTypes.FObjectDataResource(
                            (UAssetAPI.UnrealTypes.EObjectDataResourceFlags)0,
                            0,
                            -1,
                            dataSize,
                            dataSize,
                            new UAssetAPI.UnrealTypes.FPackageIndex(1),
                            0x48
                        );
                        
                        asset.DataResources.Clear();
                        asset.DataResources.Add(inlineResource);
                        mip.BulkData.Header.DataResourceIndex = targetDataResourceId;
                    }
                    
                    // Save the modified asset (first pass)
                    asset.Write(filePath);
                    
                    // Second pass: Find inline data offset and update DataResource
                    string uexpPath = Path.ChangeExtension(filePath, ".uexp");
                    if (File.Exists(uexpPath) && asset.DataResources != null && asset.DataResources.Count > 0)
                    {
                        var mip = textureExport.PlatformData?.Mips?[0];
                        if (mip?.BulkData?.Data != null && mip.BulkData.Data.Length >= 4)
                        {
                            int drIndex = mip.BulkData.Header.DataResourceIndex;
                            if (drIndex < 0 || drIndex >= asset.DataResources.Count)
                                drIndex = asset.DataResources.Count - 1;
                            
                            byte[] uexpData = File.ReadAllBytes(uexpPath);
                            byte[] searchPattern = new byte[4];
                            Array.Copy(mip.BulkData.Data, 0, searchPattern, 0, 4);
                            
                            long inlineOffset = -1;
                            for (int i = 0; i < uexpData.Length - 4; i++)
                            {
                                if (uexpData[i] == searchPattern[0] && 
                                    uexpData[i+1] == searchPattern[1] &&
                                    uexpData[i+2] == searchPattern[2] &&
                                    uexpData[i+3] == searchPattern[3])
                                {
                                    inlineOffset = i;
                                    break;
                                }
                            }
                            
                            if (inlineOffset >= 0)
                            {
                                var dr = asset.DataResources[drIndex];
                                asset.DataResources[drIndex] = new UAssetAPI.UnrealTypes.FObjectDataResource(
                                    dr.Flags,
                                    inlineOffset,
                                    dr.DuplicateSerialOffset,
                                    dr.SerialSize,
                                    dr.RawSize,
                                    dr.OuterIndex,
                                    dr.LegacyBulkDataFlags,
                                    dr.CookedIndex
                                );
                                asset.Write(filePath);
                            }
                        }
                    }
                    
                    // Patch data_resource_id in .uexp
                    if (File.Exists(uexpPath))
                    {
                        byte[] uexpBytes = File.ReadAllBytes(uexpPath);
                        int targetValue = targetDataResourceId;
                        
                        if (targetValue > 0 && targetValue < 100)
                        {
                            int firstPos = -1;
                            int secondPos = -1;
                            
                            for (int i = 100; i < Math.Min(uexpBytes.Length - 8, 300); i++)
                            {
                                int val1 = BitConverter.ToInt32(uexpBytes, i);
                                int val2 = BitConverter.ToInt32(uexpBytes, i + 4);
                                
                                if (val1 == 1 && val2 == 0 && firstPos == -1)
                                    firstPos = i + 4;
                                else if (val1 == 1 && val2 == targetValue && secondPos == -1 && firstPos >= 0)
                                    secondPos = i + 4;
                            }
                            
                            if (firstPos >= 0 && secondPos >= 0 && (secondPos - firstPos) == 64)
                            {
                                byte[] targetBytes = BitConverter.GetBytes(targetValue);
                                byte[] zeroBytes = BitConverter.GetBytes(0);
                                
                                Array.Copy(targetBytes, 0, uexpBytes, firstPos, 4);
                                Array.Copy(zeroBytes, 0, uexpBytes, secondPos, 4);
                                
                                File.WriteAllBytes(uexpPath, uexpBytes);
                            }
                        }
                    }
                    
                    // Delete .ubulk file
                    string ubulkPath = Path.ChangeExtension(filePath, ".ubulk");
                    if (File.Exists(ubulkPath))
                    {
                        File.Delete(ubulkPath);
                    }
                    
                    results.Add(new { 
                        path = filePath, 
                        success = true, 
                        message = $"Stripped {originalMipCount} -> {textureExport.MipCount}",
                        original_mips = originalMipCount,
                        new_mips = textureExport.MipCount
                    });
                    successCount++;
                    
                    Console.Error.WriteLine($"[UAssetTool] Stripped: {Path.GetFileName(filePath)} ({originalMipCount} -> {textureExport.MipCount})");
                }
                catch (Exception ex)
                {
                    results.Add(new { path = filePath, success = false, message = ex.Message });
                    errorCount++;
                    Console.Error.WriteLine($"[UAssetTool] Error processing {Path.GetFileName(filePath)}: {ex.Message}");
                }
            }
            
            Console.Error.WriteLine($"[UAssetTool] Batch complete: {successCount} stripped, {skipCount} skipped, {errorCount} errors");
            
            return new UAssetResponse
            {
                Success = true,
                Message = $"Batch processed {filePaths.Count} files: {successCount} stripped, {skipCount} skipped, {errorCount} errors",
                Data = new { 
                    total = filePaths.Count,
                    success_count = successCount,
                    skip_count = skipCount,
                    error_count = errorCount,
                    results = results
                }
            };
        }
        catch (Exception ex)
        {
            Console.Error.WriteLine($"[UAssetTool] Batch strip error: {ex.Message}");
            return new UAssetResponse { Success = false, Message = $"Batch error: {ex.Message}" };
        }
    }
    
    /// <summary>
    /// Find UE4-DDS-Tools installation path.
    /// </summary>
    private static string? FindUE4DDSTools()
    {
        Console.Error.WriteLine("[UAssetTool] Searching for UE4-DDS-Tools...");
        
        // Check relative to this executable
        string? exeDir = AppContext.BaseDirectory;
        Console.Error.WriteLine($"[UAssetTool] BaseDirectory: {exeDir}");
        
        if (!string.IsNullOrEmpty(exeDir))
        {
            // Check in ue4-dds-tools subdirectory (standard location)
            string inSubdir = Path.Combine(exeDir, "ue4-dds-tools");
            Console.Error.WriteLine($"[UAssetTool] Checking: {inSubdir}");
            if (Directory.Exists(inSubdir) && File.Exists(Path.Combine(inSubdir, "python", "python.exe")))
            {
                Console.Error.WriteLine($"[UAssetTool] Found UE4-DDS-Tools at: {inSubdir}");
                return inSubdir;
            }
            
            // Check next to executable
            if (File.Exists(Path.Combine(exeDir, "python", "python.exe")))
            {
                Console.Error.WriteLine($"[UAssetTool] Found UE4-DDS-Tools at: {exeDir}");
                return exeDir;
            }
        }
        
        // Check environment variable
        string? envPath = Environment.GetEnvironmentVariable("UE4_DDS_TOOLS_PATH");
        if (!string.IsNullOrEmpty(envPath) && Directory.Exists(envPath))
        {
            Console.Error.WriteLine($"[UAssetTool] Found UE4-DDS-Tools via env var: {envPath}");
            return envPath;
        }
        
        // Check common development paths (relative to exe or absolute)
        string[] devPaths = new[]
        {
            "../UE4-DDS-Tools",
            "../../UE4-DDS-Tools",
            "../../../uasset_toolkit/tools/UE4-DDS-Tools",
            // Absolute paths for Tauri dev environment
            @"E:\WindsurfCoding\repak_rivals-remastered\Repak_Gui-Revamped-TauriUpdate\uasset_toolkit\tools\UE4-DDS-Tools",
            @"E:\WindsurfCoding\repak_rivals-remastered\Repak_Gui-Revamped-TauriUpdate\target\debug\uassettool\ue4-dds-tools"
        };
        
        foreach (var relPath in devPaths)
        {
            string fullPath;
            if (Path.IsPathRooted(relPath))
            {
                fullPath = relPath;
            }
            else
            {
                fullPath = !string.IsNullOrEmpty(exeDir) ? Path.GetFullPath(Path.Combine(exeDir, relPath)) : relPath;
            }
            
            Console.Error.WriteLine($"[UAssetTool] Checking dev path: {fullPath}");
            if (Directory.Exists(fullPath) && File.Exists(Path.Combine(fullPath, "python", "python.exe")))
            {
                Console.Error.WriteLine($"[UAssetTool] Found UE4-DDS-Tools at dev path: {fullPath}");
                return fullPath;
            }
        }
        
        Console.Error.WriteLine("[UAssetTool] UE4-DDS-Tools NOT FOUND!");
        return null;
    }
    
    /// <summary>
    /// Run UE4-DDS-Tools command and capture output.
    /// </summary>
    private static UAssetResponse RunUE4DDSTools(string pythonExe, string mainPy, string arguments)
    {
        try
        {
            var startInfo = new System.Diagnostics.ProcessStartInfo
            {
                FileName = pythonExe,
                Arguments = $"\"{mainPy}\" {arguments}",
                UseShellExecute = false,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                CreateNoWindow = true,
                WorkingDirectory = Path.GetDirectoryName(mainPy) ?? ""
            };
            
            using var process = System.Diagnostics.Process.Start(startInfo);
            if (process == null)
                return new UAssetResponse { Success = false, Message = "Failed to start process" };
            
            string stdout = process.StandardOutput.ReadToEnd();
            string stderr = process.StandardError.ReadToEnd();
            process.WaitForExit(60000); // 60 second timeout
            
            string output = stdout + (string.IsNullOrEmpty(stderr) ? "" : "\n" + stderr);
            
            if (process.ExitCode != 0)
            {
                return new UAssetResponse 
                { 
                    Success = false, 
                    Message = $"Exit code {process.ExitCode}: {output.Trim()}"
                };
            }
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = output.Trim()
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Process error: {ex.Message}" };
        }
    }
    
    private static UAssetResponse GetTextureInfo(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            var asset = LoadAsset(filePath, usmapPath);
            asset.UseSeparateBulkDataFiles = true;
            
            var info = ExtractTextureInfo(asset);
            return new UAssetResponse { Success = true, Message = "Texture info retrieved", Data = info };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    private static Dictionary<string, object> ExtractTextureInfo(UAsset asset)
    {
        var info = new Dictionary<string, object>
        {
            ["IsTexture2D"] = false,
            ["MipGenSettings"] = "Unknown"
        };
        
        foreach (var export in asset.Exports)
        {
            if (GetExportClassName(asset, export) == "Texture2D" && export is NormalExport normalExport)
            {
                info["IsTexture2D"] = true;
                
                var properties = new List<Dictionary<string, string>>();
                foreach (var prop in normalExport.Data)
                {
                    var propInfo = new Dictionary<string, string>
                    {
                        ["Name"] = prop.Name?.Value?.Value ?? "Unknown",
                        ["Type"] = prop.GetType().Name
                    };
                    
                    if (prop is EnumPropertyData enumProp)
                        propInfo["Value"] = enumProp.Value?.Value?.Value ?? "null";
                    else if (prop is BytePropertyData byteProp)
                        propInfo["Value"] = byteProp.Value.ToString();
                    else if (prop is IntPropertyData intProp)
                        propInfo["Value"] = intProp.Value.ToString();
                    else if (prop is BoolPropertyData boolProp)
                        propInfo["Value"] = boolProp.Value.ToString();
                    else
                        propInfo["Value"] = "(complex)";
                    
                    properties.Add(propInfo);
                    
                    if (prop.Name?.Value?.Value == "MipGenSettings")
                    {
                        if (prop is EnumPropertyData ep)
                            info["MipGenSettings"] = ep.Value?.Value?.Value ?? "Unknown";
                        else if (prop is BytePropertyData bp)
                            info["MipGenSettings"] = GetMipGenSettingName(bp.Value);
                    }
                }
                info["Properties"] = properties;
                break;
            }
        }
        
        return info;
    }
    
    #endregion

    #region Mesh Operations
    
    private static UAssetResponse PatchMesh(string? filePath, string? uexpPath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (string.IsNullOrEmpty(uexpPath))
            return new UAssetResponse { Success = false, Message = "UEXP path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };
        if (!File.Exists(uexpPath))
            return new UAssetResponse { Success = false, Message = $"UEXP not found: {uexpPath}" };

        try
        {
            File.Copy(filePath, filePath + ".backup", true);
            File.Copy(uexpPath, uexpPath + ".backup", true);
            
            // TODO: Implement actual mesh patching
            return new UAssetResponse { Success = true, Message = "Mesh patch placeholder (backups created)" };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    private static UAssetResponse GetMeshInfo(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            var info = new Dictionary<string, object>
            {
                ["MaterialCount"] = 0,
                ["VertexCount"] = 0,
                ["TriangleCount"] = 0,
                ["IsSkeletalMesh"] = false
            };
            
            // TODO: Implement actual mesh info extraction
            return new UAssetResponse { Success = true, Message = "Mesh info placeholder", Data = info };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    private static UAssetResponse FixSerializeSizeJson(string? filePath, string? usmapPath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        var result = FixSerializeSize(filePath, usmapPath);
        return new UAssetResponse
        {
            Success = (bool)(result.GetType().GetProperty("success")?.GetValue(result) ?? false),
            Message = (string)(result.GetType().GetProperty("message")?.GetValue(result) ?? ""),
            Data = result
        };
    }
    
    private static object FixSerializeSize(string uassetPath, string? usmapPath)
    {
        if (string.IsNullOrEmpty(usmapPath) || !File.Exists(usmapPath))
        {
            return new { success = false, message = "USmap file required for SerializeSize fix", fixed_count = 0 };
        }

        var asset = LoadAsset(uassetPath, usmapPath);
        asset.UseSeparateBulkDataFiles = true;
        
        string uexpPath = uassetPath.Replace(".uasset", ".uexp");
        if (!File.Exists(uexpPath))
        {
            return new { success = false, message = "No .uexp file found", fixed_count = 0 };
        }

        long uexpSize = new FileInfo(uexpPath).Length;
        long headerSize = asset.Exports.Min(e => e.SerialOffset);
        var sortedExports = asset.Exports.OrderBy(e => e.SerialOffset).ToList();
        
        var fixes = new List<object>();
        int fixedCount = 0;

        for (int i = 0; i < sortedExports.Count; i++)
        {
            var export = sortedExports[i];
            long startInUexp = export.SerialOffset - headerSize;
            long endInUexp = (i < sortedExports.Count - 1) 
                ? sortedExports[i + 1].SerialOffset - headerSize 
                : uexpSize;
            
            long actualSize = endInUexp - startInUexp;
            long headerSize_current = export.SerialSize;
            
            if (actualSize != headerSize_current)
            {
                long iostorePadding = 24;
                long finalSize = actualSize + iostorePadding;
                
                fixes.Add(new
                {
                    export_name = export.ObjectName?.Value?.Value ?? $"Export_{i}",
                    old_size = headerSize_current,
                    new_size = finalSize,
                    difference = finalSize - headerSize_current
                });
                fixedCount++;
            }
        }

        return new
        {
            success = true,
            message = fixedCount > 0 ? $"Found {fixedCount} SerialSize mismatches" : "No fixes needed",
            fixed_count = fixedCount,
            fixes = fixes
        };
    }
    
    #endregion

    #region Debug Operations
    
    private static UAssetResponse DebugAssetInfo(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
            return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath))
            return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            var asset = LoadAsset(filePath, usmapPath);
            
            var info = new Dictionary<string, object>();
            
            var exports = new List<Dictionary<string, string>>();
            foreach (var export in asset.Exports)
            {
                exports.Add(new Dictionary<string, string>
                {
                    ["ExportType"] = export.GetType().Name,
                    ["ObjectName"] = export.ObjectName?.Value?.Value ?? "null",
                    ["ClassName"] = GetExportClassName(asset, export)
                });
            }
            info["Exports"] = exports;
            
            var imports = asset.Imports.Select(i => i.ObjectName?.Value?.Value ?? "null").ToList();
            info["Imports"] = imports;
            info["DetectedType"] = DetectAssetType(asset);
            
            return new UAssetResponse
            {
                Success = true,
                Message = $"Asset info for {Path.GetFileName(filePath)}",
                Data = info
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error: {ex.Message}" };
        }
    }
    
    private static void DumpAssetInfo(UAsset asset, string filePath)
    {
        Console.WriteLine($"=== Asset Dump: {Path.GetFileName(filePath)} ===");
        Console.WriteLine($"Detected Type: {DetectAssetType(asset)}");
        Console.WriteLine($"Exports: {asset.Exports.Count}");
        Console.WriteLine($"Imports: {asset.Imports.Count}");
        Console.WriteLine();
        
        Console.WriteLine("=== Exports ===");
        for (int i = 0; i < asset.Exports.Count; i++)
        {
            var export = asset.Exports[i];
            Console.WriteLine($"  [{i}] {export.ObjectName?.Value?.Value} (Class: {GetExportClassName(asset, export)})");
            Console.WriteLine($"      SerialOffset: 0x{export.SerialOffset:X}, SerialSize: {export.SerialSize}");
        }
        
        Console.WriteLine();
        Console.WriteLine("=== Imports ===");
        for (int i = 0; i < asset.Imports.Count; i++)
        {
            var import = asset.Imports[i];
            Console.WriteLine($"  [{i}] {import.ObjectName?.Value?.Value}");
        }
    }
    
    #endregion

    #region Asset Loading Helpers
    
    private static UAsset LoadAsset(string filePath, string? usmapPath)
    {
        Usmap? mappings = LoadMappings(usmapPath);
        return LoadAssetWithMappings(filePath, mappings);
    }
    
    private static UAsset LoadAssetWithMappings(string filePath, Usmap? mappings)
    {
        var asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings);
        asset.UseSeparateBulkDataFiles = true;
        return asset;
    }
    
    private static Usmap? LoadMappings(string? usmapPath)
    {
        if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
        {
            return new Usmap(usmapPath);
        }
        return null;
    }
    
    private static void WriteJsonResponse(bool success, string message, object? data = null)
    {
        var response = new UAssetResponse { Success = success, Message = message, Data = data };
        Console.WriteLine(JsonSerializer.Serialize(response));
    }
    
    #endregion
}

#region Request/Response Models

public class UAssetRequest
{
    [JsonPropertyName("action")]
    public string Action { get; set; } = "";
    
    [JsonPropertyName("file_path")]
    public string? FilePath { get; set; }
    
    [JsonPropertyName("file_paths")]
    public List<string>? FilePaths { get; set; }
    
    [JsonPropertyName("mip_gen")]
    public string? MipGen { get; set; }
    
    [JsonPropertyName("uexp_path")]
    public string? UexpPath { get; set; }
    
    [JsonPropertyName("usmap_path")]
    public string? UsmapPath { get; set; }
}

public class UAssetResponse
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }
    
    [JsonPropertyName("message")]
    public string Message { get; set; } = "";
    
    [JsonPropertyName("data")]
    public object? Data { get; set; }
}

#endregion
