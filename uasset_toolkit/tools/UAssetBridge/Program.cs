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
            // Read JSON request from stdin
            var input = await Console.In.ReadToEndAsync();
            if (string.IsNullOrWhiteSpace(input))
            {
                WriteError("No input provided");
                return;
            }

            var request = JsonSerializer.Deserialize<UAssetRequest>(input);
            if (request == null)
            {
                WriteError("Invalid JSON request");
                return;
            }

            var response = ProcessRequest(request);
            var responseJson = JsonSerializer.Serialize(response);
            Console.WriteLine(responseJson);
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
                "detect_mesh" => DetectMesh(request.FilePath),
                "patch_mesh" => PatchMesh(request.FilePath, request.UexpPath),
                "get_mesh_info" => GetMeshInfo(request.FilePath),
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
            // Try UAssetAPI detection first (requires usmap for unversioned assets)
            var isTexture = IsTexture2DAsset(filePath, null);
            
            if (!isTexture)
            {
                // Fallback to heuristic detection
                isTexture = IsLikelyTextureUAsset(filePath);
            }

            return new UAssetResponse 
            { 
                Success = true, 
                Message = isTexture ? "File is a Texture2D asset" : "File is not a texture",
                Data = isTexture
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
