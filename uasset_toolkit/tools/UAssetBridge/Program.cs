using System;
using System.IO;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Collections.Generic;
using System.Linq;
using UAssetAPI;
using UAssetAPI.UnrealTypes;
using UAssetAPI.ExportTypes;
using UAssetAPI.PropertyTypes.Objects;
using UAssetAPI.Unversioned;

namespace UAssetBridge;

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

public class Program
{
    public static async Task Main(string[] args)
    {
        try
        {
            // Interactive mode: Read line by line
            string? line;
            while ((line = await Console.In.ReadLineAsync()) != null)
            {
                if (string.IsNullOrWhiteSpace(line)) continue;

                try 
                {
                    var request = JsonSerializer.Deserialize<UAssetRequest>(line);
                    if (request == null)
                    {
                        WriteError("Invalid JSON request");
                        continue;
                    }

                    var response = ProcessRequest(request);
                    var responseJson = JsonSerializer.Serialize(response);
                    // Ensure single line output
                    Console.WriteLine(responseJson.Replace("\r", "").Replace("\n", ""));
                }
                catch (JsonException)
                {
                    WriteError("Invalid JSON format");
                }
            }
        }
        catch (Exception ex)
        {
            WriteError($"Unhandled exception: {ex.Message}");
        }
    }

    private static UAssetResponse ProcessRequest(UAssetRequest request)
    {
        try
        {
            return request.Action switch
            {
                "detect_texture" => DetectTexture(request.FilePath),
                "set_mip_gen" => SetMipGen(request.FilePath, request.MipGen),
                "get_texture_info" => GetTextureInfo(request.FilePath),
                "detect_mesh" => DetectSkeletalMesh(request.FilePath), // Backward compat: "detect_mesh" implies Skeletal for Fix Mesh
                "detect_skeletal_mesh" => DetectSkeletalMesh(request.FilePath),
                "detect_static_mesh" => DetectStaticMesh(request.FilePath),
                "patch_mesh" => PatchMesh(request.FilePath, request.UexpPath),
                "get_mesh_info" => GetMeshInfo(request.FilePath),
                // Batch detection - processes all files in parallel, returns true if any match
                "batch_detect_skeletal_mesh" => BatchDetectAssetClass(request.FilePaths, "SkeletalMesh"),
                "batch_detect_static_mesh" => BatchDetectAssetClass(request.FilePaths, "StaticMesh"),
                "batch_detect_texture" => BatchDetectTexture(request.FilePaths),
                // Debug action to dump asset info
                "debug_asset_info" => DebugAssetInfo(request.FilePath),
                _ => new UAssetResponse 
                { 
                    Success = false, 
                    Message = $"Unknown action: {request.Action}" 
                }
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error processing request: {ex.Message}" 
            };
        }
    }

    private static UAssetResponse DebugAssetInfo(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath)) return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath)) return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            Usmap? mappings = null;
            if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
            {
                mappings = new Usmap(usmapPath);
            }

            // Pass mappings during construction for unversioned assets
            var asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings);
            
            var info = new Dictionary<string, object>();
            
            // Collect export info
            var exports = new List<Dictionary<string, string>>();
            foreach (var export in asset.Exports)
            {
                var exportInfo = new Dictionary<string, string>
                {
                    ["ExportType"] = export.GetType().Name,
                    ["ObjectName"] = export.ObjectName?.Value?.Value ?? "null",
                    ["ClassIndex"] = export.ClassIndex.ToString(),
                    ["IsImport"] = export.ClassIndex.IsImport().ToString()
                };
                
                if (export.ClassIndex.IsImport())
                {
                    var import = export.ClassIndex.ToImport(asset);
                    if (import != null)
                    {
                        exportInfo["ImportClassName"] = import.ObjectName?.Value?.Value ?? "null";
                        exportInfo["ImportClassPackage"] = import.ClassPackage?.Value?.Value ?? "null";
                    }
                }
                
                exports.Add(exportInfo);
            }
            info["Exports"] = exports;
            
            // Collect all imports
            var imports = new List<string>();
            foreach (var imp in asset.Imports)
            {
                imports.Add(imp.ObjectName?.Value?.Value ?? "null");
            }
            info["AllImports"] = imports;
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = $"Asset info for {Path.GetFileName(filePath)}",
                Data = info
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse { Success = false, Message = $"Error reading asset: {ex.Message}" };
        }
    }

    private static UAssetResponse DetectSkeletalMesh(string? filePath)
    {
        return DetectAssetClass(filePath, "SkeletalMesh");
    }

    private static UAssetResponse DetectStaticMesh(string? filePath)
    {
        return DetectAssetClass(filePath, "StaticMesh");
    }

    private static UAssetResponse DetectAssetClass(string? filePath, string targetClass)
    {
        if (string.IsNullOrEmpty(filePath)) return new UAssetResponse { Success = false, Message = "File path required" };
        if (!File.Exists(filePath)) return new UAssetResponse { Success = false, Message = $"File not found: {filePath}" };

        try
        {
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            Usmap? mappings = null;
            if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
            {
                mappings = new Usmap(usmapPath);
            }

            // Pass mappings during construction for unversioned assets
            var asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings);
            
            bool isMatch = IsAssetClass(asset, targetClass);
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = isMatch ? $"File is {targetClass}" : $"File is not {targetClass}",
                Data = isMatch
            };
        }
        catch (Exception ex)
        {
             return new UAssetResponse { Success = false, Message = $"Error reading uasset: {ex.Message}" };
        }
    }

    private static bool IsAssetClass(UAsset asset, string targetClass)
    {
        foreach (var export in asset.Exports)
        {
            // Method 1: Check export type name directly (most reliable for UE5)
            string exportTypeName = export.GetType().Name;
            if (exportTypeName.Contains(targetClass, StringComparison.OrdinalIgnoreCase))
            {
                return true;
            }
            
            // Method 2: Check ClassIndex import
            if (export.ClassIndex.IsImport())
            {
                var import = export.ClassIndex.ToImport(asset);
                if (import != null)
                {
                    string className = import.ObjectName?.Value?.Value ?? "";
                    if (className.Contains(targetClass, StringComparison.OrdinalIgnoreCase))
                    {
                        return true;
                    }
                }
            }
            
            // Method 3: Check export's ObjectName for mesh naming conventions
            string? objectName = export.ObjectName?.Value?.Value;
            if (!string.IsNullOrEmpty(objectName))
            {
                if (targetClass.Equals("SkeletalMesh", StringComparison.OrdinalIgnoreCase))
                {
                    // SK_ prefix is standard UE naming for skeletal meshes
                    if (objectName.StartsWith("SK_", StringComparison.OrdinalIgnoreCase))
                    {
                        return true;
                    }
                }
                else if (targetClass.Equals("StaticMesh", StringComparison.OrdinalIgnoreCase))
                {
                    // SM_ prefix is standard UE naming for static meshes
                    if (objectName.StartsWith("SM_", StringComparison.OrdinalIgnoreCase))
                    {
                        return true;
                    }
                }
            }
            
            // Method 4: Check all imports in the asset for class references
            foreach (var imp in asset.Imports)
            {
                string impName = imp.ObjectName?.Value?.Value ?? "";
                if (impName.Equals(targetClass, StringComparison.OrdinalIgnoreCase) ||
                    impName.Equals("U" + targetClass, StringComparison.OrdinalIgnoreCase))
                {
                    return true;
                }
            }
        }
        return false;
    }
    
    // Check by filename pattern when UAssetAPI parsing fails
    private static bool IsAssetClassByFilename(string filePath, string targetClass)
    {
        string fileName = Path.GetFileNameWithoutExtension(filePath);
        
        if (targetClass.Equals("SkeletalMesh", StringComparison.OrdinalIgnoreCase))
        {
            return fileName.StartsWith("SK_", StringComparison.OrdinalIgnoreCase);
        }
        else if (targetClass.Equals("StaticMesh", StringComparison.OrdinalIgnoreCase))
        {
            return fileName.StartsWith("SM_", StringComparison.OrdinalIgnoreCase);
        }
        
        return false;
    }

    // Batch detection - processes files in parallel for speed
    private static UAssetResponse BatchDetectAssetClass(List<string>? filePaths, string targetClass)
    {
        if (filePaths == null || filePaths.Count == 0)
            return new UAssetResponse { Success = false, Message = "file_paths required" };

        string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
        Usmap? mappings = null;
        if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
        {
            mappings = new Usmap(usmapPath);
        }

        // DEBUG: Log info about SK_ files to find out actual class names
        var debugInfo = new List<string>();
        debugInfo.Add($"USMAP_PATH env: {usmapPath ?? "NOT SET"}");
        debugInfo.Add($"USMAP exists: {(usmapPath != null && File.Exists(usmapPath))}");
        debugInfo.Add($"Mappings loaded: {mappings != null}");
        
        foreach (var filePath in filePaths)
        {
            if (!File.Exists(filePath)) continue;
            string fileName = Path.GetFileName(filePath);
            if (fileName.StartsWith("SK_", StringComparison.OrdinalIgnoreCase))
            {
                // Check for associated .uexp file
                string uexpPath = Path.ChangeExtension(filePath, ".uexp");
                bool hasUexp = File.Exists(uexpPath);
                debugInfo.Add($"[{fileName}] uexp exists: {hasUexp} at {uexpPath}");
                
                // Check file size and first bytes for debugging
                try
                {
                    var fileBytes = File.ReadAllBytes(filePath);
                    debugInfo.Add($"[{fileName}] Size: {fileBytes.Length} bytes");
                    debugInfo.Add($"[{fileName}] First 16 bytes: {BitConverter.ToString(fileBytes.Take(16).ToArray())}");
                    
                    // Check magic number
                    if (fileBytes.Length >= 4)
                    {
                        uint magic = BitConverter.ToUInt32(fileBytes, 0);
                        debugInfo.Add($"[{fileName}] Magic: 0x{magic:X8} (expected 0x9E2A83C1 for uasset)");
                    }
                }
                catch (Exception ex)
                {
                    debugInfo.Add($"[{fileName}] Read bytes error: {ex.Message}");
                }
                
                // Try loading with SkipParsingExports flag
                UAsset? asset = null;
                try
                {
                    asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings, CustomSerializationFlags.SkipParsingExports);
                    debugInfo.Add($"[{fileName}] Loaded with SkipParsingExports");
                }
                catch (Exception ex1)
                {
                    debugInfo.Add($"[{fileName}] SkipParsingExports failed: {ex1.Message}");
                    
                    try
                    {
                        asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings);
                        debugInfo.Add($"[{fileName}] Loaded normally");
                    }
                    catch (Exception ex2)
                    {
                        debugInfo.Add($"[{fileName}] Normal load failed: {ex2.Message}");
                    }
                }
                
                if (asset == null)
                {
                    debugInfo.Add($"[{fileName}] Failed to load");
                    continue;
                }

                try
                {
                    foreach (var export in asset.Exports)
                    {
                        string exportType = export.GetType().Name;
                        string? objName = export.ObjectName?.Value?.Value;
                        string importClass = "";
                        if (export.ClassIndex.IsImport())
                        {
                            var imp = export.ClassIndex.ToImport(asset);
                            importClass = imp?.ObjectName?.Value?.Value ?? "";
                        }
                        debugInfo.Add($"[{fileName}] Export: {exportType}, Obj: {objName}, ImportClass: {importClass}");
                    }
                    
                    foreach (var imp in asset.Imports)
                    {
                        debugInfo.Add($"[{fileName}] Import: {imp.ObjectName?.Value?.Value}");
                    }
                }
                catch (Exception ex)
                {
                    debugInfo.Add($"[{fileName}] Error: {ex.Message}");
                }
            }
        }
        
        // Write debug info to a file for analysis
        if (debugInfo.Count > 0)
        {
            string debugPath = Path.Combine(Path.GetTempPath(), "uasset_debug.txt");
            File.WriteAllLines(debugPath, debugInfo);
        }

        // Process files in parallel for maximum speed
        // Use SkipParsingExports for faster detection (only reads headers + imports)
        bool foundMatch = filePaths.AsParallel().Any(filePath =>
        {
            if (!File.Exists(filePath)) return false;
            try
            {
                // First try with SkipParsingExports (fast, works without full parsing)
                var asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings, CustomSerializationFlags.SkipParsingExports);
                return IsAssetClass(asset, targetClass);
            }
            catch
            {
                // Fallback to normal loading
                try
                {
                    var asset = new UAsset(filePath, EngineVersion.VER_UE5_3, mappings);
                    return IsAssetClass(asset, targetClass);
                }
                catch
                {
                    return false;
                }
            }
        });

        return new UAssetResponse
        {
            Success = true,
            Message = foundMatch ? $"Found {targetClass} in batch" : $"No {targetClass} found in batch",
            Data = foundMatch
        };
    }

    private static UAssetResponse BatchDetectTexture(List<string>? filePaths)
    {
        if (filePaths == null || filePaths.Count == 0)
            return new UAssetResponse { Success = false, Message = "file_paths required" };

        // Get usmap path from environment
        string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");

        // Process files in parallel for maximum speed
        // Only return true if texture needs MipGen fixing (not already NoMipmaps)
        bool foundMatch = filePaths.AsParallel().Any(filePath =>
        {
            if (!File.Exists(filePath)) return false;
            try
            {
                // First check if it needs fixing using accurate detection
                bool needsFix = IsTextureNeedingMipGenFix(filePath, usmapPath);
                if (needsFix) return true;
                
                // Fallback to heuristic for files that can't be loaded
                // But heuristic can't check MipGenSettings, so be conservative
                return false;
            }
            catch
            {
                return false;
            }
        });

        return new UAssetResponse
        {
            Success = true,
            Message = foundMatch ? "Found Texture needing MipGen fix in batch" : "No Texture needing fix found in batch",
            Data = foundMatch
        };
    }

    private static UAssetResponse DetectTexture(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "File path is required" 
            };
        }

        if (!File.Exists(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"File not found: {filePath}" 
            };
        }

        try
        {
            // Get usmap path from environment
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            
            // Check if texture needs MipGen fixing (not already NoMipmaps)
            var needsFix = IsTextureNeedingMipGenFix(filePath, usmapPath);

            return new UAssetResponse 
            { 
                Success = true, 
                Message = needsFix ? "File is a Texture2D asset needing MipGen fix" : "File is not a texture or already has NoMipmaps",
                Data = needsFix
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error reading uasset file: {ex.Message}" 
            };
        }
    }

    private static bool IsLikelyTextureUAsset(string filePath)
    {
        try
        {
            // Basic heuristic: check if filename suggests it's a texture
            var fileName = Path.GetFileNameWithoutExtension(filePath).ToLowerInvariant();
            
            // Common texture naming patterns
            var textureIndicators = new[] { "tex", "texture", "diffuse", "normal", "specular", "roughness", "metallic", "albedo", "basecolor", "t_" };
            
            foreach (var indicator in textureIndicators)
            {
                if (fileName.Contains(indicator))
                {
                    return true;
                }
            }

            // Check file size - textures are typically larger
            var fileInfo = new FileInfo(filePath);
            if (fileInfo.Length > 10000) // 10KB threshold
            {
                return true;
            }

            return false;
        }
        catch
        {
            return false;
        }
    }

    private static UAssetResponse SetMipGen(string? filePath, string? mipGen)
    {
        if (string.IsNullOrEmpty(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "File path is required" 
            };
        }

        if (string.IsNullOrEmpty(mipGen))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "Mip gen setting is required" 
            };
        }

        if (!File.Exists(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"File not found: {filePath}" 
            };
        }

        try
        {
            // Get usmap path from environment or request
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            
            // Load mappings if available
            Usmap? mappings = null;
            if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
            {
                mappings = new Usmap(usmapPath);
            }
            
            // Load the asset
            var asset = new UAsset(filePath, EngineVersion.VER_UE5_3);
            if (mappings != null)
            {
                asset.Mappings = mappings;
            }
            asset.UseSeparateBulkDataFiles = true;
            
            // Verify it's a Texture2D
            if (!IsTexture2DAsset(asset))
            {
                return new UAssetResponse
                {
                    Success = false,
                    Message = "Asset is not a Texture2D"
                };
            }
            
            // Parse the MipGen setting
            byte mipGenValue = mipGen.ToLower() switch
            {
                "nomipmaps" => 13, // TMGS_NoMipmaps
                "simpleaverage" => 1, // TMGS_SimpleAverage
                "fromtexturegroup" => 0, // TMGS_FromTextureGroup
                _ => throw new Exception($"Unknown MipGen setting: {mipGen}")
            };
            
            // Patch the MipGenSettings property
            bool patched = PatchTextureMipGenSettings(asset, mipGenValue);
            
            if (!patched)
            {
                return new UAssetResponse
                {
                    Success = false,
                    Message = "Could not find MipGenSettings property to patch"
                };
            }
            
            // Create backup
            var backupPath = filePath + ".backup";
            File.Copy(filePath, backupPath, true);
            
            // Write the modified asset
            asset.Write(filePath);
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = $"Successfully set MipGenSettings to {mipGen} (value: {mipGenValue})" 
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error modifying uasset file: {ex.Message}" 
            };
        }
    }

    private static UAssetResponse GetTextureInfo(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "File path is required" 
            };
        }

        if (!File.Exists(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"File not found: {filePath}" 
            };
        }

        try
        {
            // Get usmap path from environment
            string? usmapPath = Environment.GetEnvironmentVariable("USMAP_PATH");
            
            // Load mappings if available
            Usmap? mappings = null;
            if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
            {
                mappings = new Usmap(usmapPath);
            }
            
            // Load the asset
            var asset = new UAsset(filePath, EngineVersion.VER_UE5_3);
            if (mappings != null)
            {
                asset.Mappings = mappings;
            }
            asset.UseSeparateBulkDataFiles = true;
            
            // Extract texture info
            var textureInfo = ExtractTextureInfo(asset);
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = "Texture info retrieved",
                Data = textureInfo
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error reading texture info: {ex.Message}" 
            };
        }
    }

    private static UAssetResponse DetectMesh(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "File path is required" 
            };
        }

        if (!File.Exists(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"File not found: {filePath}" 
            };
        }

        try
        {
            // Heuristic mesh detection based on filename patterns and path
            var isMesh = IsLikelyMeshUAsset(filePath);

            return new UAssetResponse 
            { 
                Success = true, 
                Message = isMesh ? "File is likely a mesh" : "File is not a mesh",
                Data = isMesh
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error reading uasset file: {ex.Message}" 
            };
        }
    }

    private static bool IsLikelyMeshUAsset(string filePath)
    {
        try
        {
            var fileName = Path.GetFileNameWithoutExtension(filePath).ToLowerInvariant();
            var pathStr = filePath.ToLowerInvariant();
            
            // Common mesh naming patterns and path indicators
            var meshIndicators = new[] { 
                "mesh", "sk_", "sm_", "skeletal", "static", "character", "weapon", "armor", 
                "body", "head", "hair", "face", "/meshes/", "\\meshes\\", "_mesh", "model" 
            };
            
            foreach (var indicator in meshIndicators)
            {
                if (fileName.Contains(indicator) || pathStr.Contains(indicator))
                {
                    return true;
                }
            }

            // Check file size - meshes are typically larger than textures
            var fileInfo = new FileInfo(filePath);
            if (fileInfo.Length > 50000) // 50KB threshold for meshes
            {
                return true;
            }

            return false;
        }
        catch
        {
            return false;
        }
    }

    private static UAssetResponse PatchMesh(string? filePath, string? uexpPath)
    {
        if (string.IsNullOrEmpty(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "File path is required" 
            };
        }

        if (string.IsNullOrEmpty(uexpPath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "UEXP path is required" 
            };
        }

        if (!File.Exists(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"UAsset file not found: {filePath}" 
            };
        }

        if (!File.Exists(uexpPath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"UEXP file not found: {uexpPath}" 
            };
        }

        try
        {
            // Create backups
            var uassetBackup = filePath + ".backup";
            var uexpBackup = uexpPath + ".backup";
            File.Copy(filePath, uassetBackup, true);
            File.Copy(uexpPath, uexpBackup, true);
            
            // TODO: Implement actual mesh patching using UAssetAPI
            // For now, just return success to allow testing of the Rust integration
            // The actual mesh patching logic from uasset_mesh_patch_rivals can be integrated here
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = "Placeholder: Would patch mesh materials (backups created)" 
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error patching mesh: {ex.Message}" 
            };
        }
    }

    private static UAssetResponse GetMeshInfo(string? filePath)
    {
        if (string.IsNullOrEmpty(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = "File path is required" 
            };
        }

        if (!File.Exists(filePath))
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"File not found: {filePath}" 
            };
        }

        try
        {
            var meshInfo = new Dictionary<string, object>
            {
                ["MaterialCount"] = 0,
                ["VertexCount"] = 0,
                ["TriangleCount"] = 0,
                ["IsSkeletalMesh"] = false
            };

            // TODO: Implement actual mesh info extraction with UAssetAPI
            // For now, return placeholder data
            
            return new UAssetResponse 
            { 
                Success = true, 
                Message = "Placeholder mesh info retrieved",
                Data = meshInfo
            };
        }
        catch (Exception ex)
        {
            return new UAssetResponse 
            { 
                Success = false, 
                Message = $"Error reading mesh info: {ex.Message}" 
            };
        }
    }

    private static void WriteError(string message)
    {
        var errorResponse = new UAssetResponse 
        { 
            Success = false, 
            Message = message 
        };
        var errorJson = JsonSerializer.Serialize(errorResponse);
        Console.WriteLine(errorJson);
    }
    
    // Helper methods for Texture2D detection and patching
    
    private static bool IsTexture2DAsset(string filePath, string? usmapPath)
    {
        try
        {
            Usmap? mappings = null;
            if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
            {
                mappings = new Usmap(usmapPath);
            }
            
            var asset = new UAsset(filePath, EngineVersion.VER_UE5_3);
            if (mappings != null)
            {
                asset.Mappings = mappings;
            }
            
            return IsTexture2DAsset(asset);
        }
        catch
        {
            return false;
        }
    }
    
    private static bool IsTexture2DAsset(UAsset asset)
    {
        foreach (var export in asset.Exports)
        {
            if (export.ClassIndex.IsImport())
            {
                var import = export.ClassIndex.ToImport(asset);
                if (import != null)
                {
                    string className = import.ObjectName?.Value?.Value ?? "";
                    if (className.Equals("Texture2D", StringComparison.OrdinalIgnoreCase))
                    {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    
    /// <summary>
    /// Checks if a texture asset needs MipGen fixing.
    /// Returns true only if:
    /// 1. The asset is a Texture2D
    /// 2. Its MipGenSettings is NOT already set to NoMipmaps (13)
    /// </summary>
    private static bool IsTextureNeedingMipGenFix(string filePath, string? usmapPath)
    {
        try
        {
            Usmap? mappings = null;
            if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
            {
                mappings = new Usmap(usmapPath);
            }
            
            var asset = new UAsset(filePath, EngineVersion.VER_UE5_3);
            asset.UseSeparateBulkDataFiles = true;
            if (mappings != null)
            {
                asset.Mappings = mappings;
            }
            
            return IsTextureNeedingMipGenFix(asset);
        }
        catch
        {
            return false;
        }
    }
    
    /// <summary>
    /// Checks if a loaded asset needs MipGen fixing.
    /// Returns true only if it's a Texture2D with MipGenSettings != NoMipmaps.
    /// </summary>
    private static bool IsTextureNeedingMipGenFix(UAsset asset)
    {
        foreach (var export in asset.Exports)
        {
            if (export.ClassIndex.IsImport())
            {
                var import = export.ClassIndex.ToImport(asset);
                if (import != null && import.ObjectName?.Value?.Value == "Texture2D")
                {
                    // Found a Texture2D, now check its MipGenSettings
                    if (export is NormalExport normalExport)
                    {
                        foreach (var property in normalExport.Data)
                        {
                            if (property.Name?.Value?.Value == "MipGenSettings")
                            {
                                // Property exists, check if it's already NoMipmaps
                                if (property is EnumPropertyData enumProp)
                                {
                                    string mipGenValue = enumProp.Value?.Value?.Value ?? "";
                                    // If already NoMipmaps, no fix needed
                                    if (mipGenValue.Equals("TMGS_NoMipmaps", StringComparison.OrdinalIgnoreCase))
                                    {
                                        return false;
                                    }
                                    // Otherwise, needs fixing
                                    return true;
                                }
                                else if (property is BytePropertyData byteProp)
                                {
                                    // 13 = TMGS_NoMipmaps
                                    if (byteProp.Value == 13)
                                    {
                                        return false;
                                    }
                                    return true;
                                }
                            }
                        }
                        // MipGenSettings property not found - using default (FromTextureGroup)
                        // This needs fixing
                        return true;
                    }
                }
            }
        }
        return false;
    }
    
    private static bool PatchTextureMipGenSettings(UAsset asset, byte mipGenValue)
    {
        bool patched = false;
        
        foreach (var export in asset.Exports)
        {
            if (export is NormalExport normalExport)
            {
                // Check if this export is actually a Texture2D
                // We verify this by checking its ClassIndex
                if (export.ClassIndex.IsImport())
                {
                    var import = export.ClassIndex.ToImport(asset);
                    if (import == null || !import.ObjectName.Value.Value.Equals("Texture2D", StringComparison.OrdinalIgnoreCase))
                    {
                        continue;
                    }
                }
                else 
                {
                    // If it's not an import, it might be the main export, but let's rely on the caller filtering
                    // or just being safe.
                    // The caller (ProcessRequest -> SetMipGen) calls IsTexture2DAsset(asset) first which checks if *any* export is Texture2D.
                    // But here we are iterating ALL exports. We should be careful.
                    // Typically the main export is the Texture2D.
                }

                bool found = false;
                foreach (var property in normalExport.Data)
                {
                    if (property.Name?.Value?.Value == "MipGenSettings")
                    {
                        if (property is EnumPropertyData enumProp)
                        {
                            // Patch the enum value
                            enumProp.Value = new FName(asset, GetMipGenSettingName(mipGenValue));
                            patched = true;
                        }
                        else if (property is BytePropertyData byteProp)
                        {
                            // Some versions store it as a byte
                            byteProp.Value = mipGenValue;
                            patched = true;
                        }
                        found = true;
                        break;
                    }
                }

                if (!found)
                {
                    // Property missing (likely using default), so we must add it.
                    // UE5 uses EnumProperty for MipGenSettings.
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
    
    private static string GetMipGenSettingName(byte value)
    {
        return value switch
        {
            0 => "TMGS_FromTextureGroup",
            1 => "TMGS_SimpleAverage",
            13 => "TMGS_NoMipmaps",
            _ => $"TMGS_Unknown_{value}"
        };
    }
    
    private static Dictionary<string, object> ExtractTextureInfo(UAsset asset)
    {
        var info = new Dictionary<string, object>
        {
            ["MipGenSettings"] = "Unknown",
            ["IsTexture2D"] = false
        };
        
        foreach (var export in asset.Exports)
        {
            if (export.ClassIndex.IsImport())
            {
                var import = export.ClassIndex.ToImport(asset);
                if (import != null && import.ObjectName?.Value?.Value == "Texture2D")
                {
                    info["IsTexture2D"] = true;
                    
                    if (export is NormalExport normalExport)
                    {
                        foreach (var property in normalExport.Data)
                        {
                            if (property.Name?.Value?.Value == "MipGenSettings")
                            {
                                if (property is EnumPropertyData enumProp)
                                {
                                    info["MipGenSettings"] = enumProp.Value?.Value?.Value ?? "Unknown";
                                }
                                else if (property is BytePropertyData byteProp)
                                {
                                    info["MipGenSettings"] = GetMipGenSettingName(byteProp.Value);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        return info;
    }
}
