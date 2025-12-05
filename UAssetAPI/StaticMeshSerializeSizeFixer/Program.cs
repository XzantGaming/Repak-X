#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using UAssetAPI;
using UAssetAPI.UnrealTypes;
using UAssetAPI.ExportTypes;
using UAssetAPI.Unversioned;

namespace StaticMeshSerializeSizeFixer
{
    class Program
    {
        static int Main(string[] args)
        {
            if (args.Length < 1)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer <command> <args>");
                Console.Error.WriteLine("");
                Console.Error.WriteLine("Commands:");
                Console.Error.WriteLine("  detect <uasset_path> [usmap_path]       - Detect asset type (static_mesh, skeletal_mesh, material_instance, or other)");
                Console.Error.WriteLine("  fix <uasset_path> [usmap_path]          - Fix SerializeSize for Static Mesh assets");
                Console.Error.WriteLine("  batch_detect <directory> [usmap_path]   - Detect all .uasset files in directory");
                return 1;
            }

            string command = args[0].ToLower();

            try
            {
                return command switch
                {
                    "detect" => DetectAssetType(args),
                    "fix" => FixStaticMeshSerializeSize(args),
                    "batch_detect" => BatchDetectAssets(args),
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

        static int DetectAssetType(string[] args)
        {
            if (args.Length < 2)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer detect <uasset_path> [usmap_path]");
                return 1;
            }

            string uassetPath = args[1];
            string? usmapPath = args.Length > 2 ? args[2] : null;

            if (!File.Exists(uassetPath))
            {
                Console.Error.WriteLine($"File not found: {uassetPath}");
                return 1;
            }

            try
            {
                Console.Error.WriteLine("=== StaticMeshSerializeSizeFixer v2.0 - DEBUG BUILD ===");
                Console.Error.Flush();
                Console.Error.WriteLine($"[DEBUG] Starting detection for: {uassetPath}");
                Console.Error.WriteLine($"[DEBUG] USmap path: {usmapPath ?? "NULL"}");
                Console.Error.Flush();
                
                // Load mappings FIRST (required for unversioned assets)
                Usmap? mappings = null;
                if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
                {
                    Console.Error.WriteLine($"[DEBUG] Loading USmap from: {usmapPath}");
                    mappings = new Usmap(usmapPath);
                    Console.Error.WriteLine($"[DEBUG] USmap loaded successfully");
                }
                else
                {
                    Console.Error.WriteLine("Warning: No usmap file provided. Detection may be limited for unversioned assets.");
                }
                
                Console.Error.WriteLine($"[DEBUG] Creating UAsset instance...");
                // For unversioned assets, use UE5.3 as base version and let mappings override
                var asset = new UAsset(uassetPath, EngineVersion.VER_UE5_3);
                
                // Set mappings BEFORE any Read operations
                if (mappings != null)
                {
                    asset.Mappings = mappings;
                    Console.Error.WriteLine($"[DEBUG] Mappings set: YES");
                }
                else
                {
                    Console.Error.WriteLine($"[DEBUG] Mappings set: NO");
                }
                
                asset.UseSeparateBulkDataFiles = true;
                Console.Error.WriteLine($"[DEBUG] Asset loaded and mappings applied");
                
                Console.Error.WriteLine($"[DEBUG] Determining asset type...");
                string assetType = DetermineAssetType(asset);
                Console.Error.WriteLine($"[DEBUG] Asset type: {assetType}");
                
                // Output as JSON for easy parsing
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
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Failed to detect asset type: {ex.Message}");
                Console.Error.WriteLine($"[DEBUG] Exception type: {ex.GetType().Name}");
                Console.Error.WriteLine($"[DEBUG] Stack trace: {ex.StackTrace}");
                if (ex.InnerException != null)
                {
                    Console.Error.WriteLine($"[DEBUG] Inner exception: {ex.InnerException.Message}");
                    Console.Error.WriteLine($"[DEBUG] Inner stack trace: {ex.InnerException.StackTrace}");
                }
                return 1;
            }
        }

        static string DetermineAssetType(UAsset asset)
        {
            Console.Error.WriteLine($"[DEBUG] DetermineAssetType: Checking {asset.Exports.Count} exports");
            
            // Check exports for class type
            for (int i = 0; i < asset.Exports.Count; i++)
            {
                var export = asset.Exports[i];
                string className = "Unknown";
                
                Console.Error.WriteLine($"[DEBUG] Export {i}: ClassIndex.IsImport()={export.ClassIndex.IsImport()}, ClassIndex.Index={export.ClassIndex.Index}");
                
                if (export.ClassIndex.IsImport())
                {
                    // For negative indices (imports), use ToImport to get the actual import
                    var import = export.ClassIndex.ToImport(asset);
                    if (import != null)
                    {
                        className = import.ObjectName?.Value?.Value ?? "Unknown";
                        Console.Error.WriteLine($"[DEBUG]   ClassName from import: {className}");
                        Console.Error.WriteLine($"[DEBUG]   Import.ClassPackage: {import.ClassPackage?.Value?.Value}");
                    }
                    else
                    {
                        Console.Error.WriteLine($"[DEBUG]   ToImport returned null!");
                    }
                }
                else
                {
                    Console.Error.WriteLine($"[DEBUG]   ClassIndex is not an import");
                }

                // Accurate detection based on UE class names
                if (className.Equals("StaticMesh", StringComparison.OrdinalIgnoreCase))
                {
                    Console.Error.WriteLine($"[DEBUG] MATCH: StaticMesh");
                    return "static_mesh";
                }
                else if (className.Equals("SkeletalMesh", StringComparison.OrdinalIgnoreCase))
                {
                    Console.Error.WriteLine($"[DEBUG] MATCH: SkeletalMesh");
                    return "skeletal_mesh";
                }
                else if (className.Equals("MaterialInstanceConstant", StringComparison.OrdinalIgnoreCase) ||
                         className.Equals("MaterialInstance", StringComparison.OrdinalIgnoreCase))
                {
                    Console.Error.WriteLine($"[DEBUG] MATCH: MaterialInstance");
                    return "material_instance";
                }
                else if (className.Equals("Texture2D", StringComparison.OrdinalIgnoreCase))
                {
                    Console.Error.WriteLine($"[DEBUG] MATCH: Texture2D");
                    return "texture";
                }
            }

            Console.Error.WriteLine($"[DEBUG] No match found, returning 'other'");
            return "other";
        }

        static int FixStaticMeshSerializeSize(string[] args)
        {
            if (args.Length < 2)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer fix <uasset_path> [usmap_path]");
                return 1;
            }

            string uassetPath = args[1];
            string? usmapPath = args.Length > 2 ? args[2] : null;

            if (!File.Exists(uassetPath))
            {
                Console.Error.WriteLine($"File not found: {uassetPath}");
                return 1;
            }

            try
            {
                // Load mappings FIRST (REQUIRED for unversioned assets)
                Usmap? mappings = null;
                if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
                {
                    mappings = new Usmap(usmapPath);
                    Console.Error.WriteLine($"[DEBUG] Loaded USmap: {usmapPath}");
                }
                else
                {
                    Console.Error.WriteLine("Error: usmap file is required for unversioned assets");
                    return 1;
                }
                
                Console.Error.WriteLine($"[DEBUG] Loading asset: {uassetPath}");
                
                // For unversioned assets, use UE5.3 as base version and let mappings override
                var asset = new UAsset(uassetPath, EngineVersion.VER_UE5_3);
                
                // Set mappings BEFORE any operations
                asset.Mappings = mappings;
                asset.UseSeparateBulkDataFiles = true;
                Console.Error.WriteLine($"[DEBUG] Asset loaded with UE5.3 + mappings");

                // Calculate correct SerialSize from ACTUAL .uexp data
                // DO NOT modify the .uexp - only patch the .uasset header!
                int exportCount = asset.Exports.Count;
                Console.Error.WriteLine($"[DEBUG] Asset has {exportCount} exports");

                // Get the .uexp file path
                string uexpPath = uassetPath.Replace(".uasset", ".uexp");
                if (!File.Exists(uexpPath))
                {
                    Console.Error.WriteLine($"[DEBUG] No .uexp file found at {uexpPath}");
                    Console.WriteLine(JsonSerializer.Serialize(new
                    {
                        success = false,
                        message = "No .uexp file found",
                        fixed_count = 0
                    }));
                    return 0;
                }

                long uexpSize = new FileInfo(uexpPath).Length;
                Console.Error.WriteLine($"[DEBUG] .uexp file size: {uexpSize} bytes");

                // Calculate the header size (offset to first export in .uexp)
                long headerSize = asset.Exports.Min(e => e.SerialOffset);
                Console.Error.WriteLine($"[DEBUG] Header size (first export offset): {headerSize}");

                // Sort exports by SerialOffset to calculate sizes
                var sortedExports = asset.Exports.OrderBy(e => e.SerialOffset).ToList();
                
                var fixes = new List<object>();
                int fixedCount = 0;

                for (int i = 0; i < sortedExports.Count; i++)
                {
                    var export = sortedExports[i];
                    long startInUexp = export.SerialOffset - headerSize;
                    long endInUexp;
                    
                    if (i < sortedExports.Count - 1)
                    {
                        // Next export's start is this export's end
                        endInUexp = sortedExports[i + 1].SerialOffset - headerSize;
                    }
                    else
                    {
                        // Last export goes to end of .uexp file
                        endInUexp = uexpSize;
                    }
                    
                    long actualSizeInUexp = endInUexp - startInUexp;
                    long headerSize_current = export.SerialSize;
                    
                    Console.Error.WriteLine($"[DEBUG] Export {export.ObjectName}:");
                    Console.Error.WriteLine($"[DEBUG]   Header SerialSize: {headerSize_current}");
                    Console.Error.WriteLine($"[DEBUG]   Actual size in .uexp: {actualSizeInUexp}");
                    Console.Error.WriteLine($"[DEBUG]   Difference: {actualSizeInUexp - headerSize_current}");
                    
                    // ONLY patch if there's a real mismatch between header and .uexp
                    // If they match, the export doesn't need fixing
                    if (actualSizeInUexp != headerSize_current)
                    {
                        // IoStore adds +24 padding, so final size = actualSizeInUexp + 24
                        long iostorePadding = 24;
                        long finalSize = actualSizeInUexp + iostorePadding;
                        
                        Console.Error.WriteLine($"[DEBUG]   -> MISMATCH! Adding IoStore padding: +{iostorePadding}");
                        Console.Error.WriteLine($"[DEBUG]   -> Final size: {finalSize}");
                        
                        fixes.Add(new
                        {
                            export_name = export.ObjectName?.Value?.Value ?? $"Export_{i}",
                            old_size = headerSize_current,
                            new_size = finalSize,
                            difference = finalSize - headerSize_current
                        });
                        fixedCount++;
                    }
                    else
                    {
                        Console.Error.WriteLine($"[DEBUG]   -> OK (header matches .uexp)");
                    }
                }

                Console.WriteLine(JsonSerializer.Serialize(new
                {
                    success = true,
                    message = fixedCount > 0 ? $"Found {fixedCount} SerialSize mismatches to fix" : "No fixes needed",
                    fixed_count = fixedCount,
                    fixes = fixes
                }, new JsonSerializerOptions { WriteIndented = true }));

                return 0;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Failed to fix SerializeSize: {ex.Message}");
                return 1;
            }
        }

        static long CalculateExportSerializedSize(UAsset asset, Export export)
        {
            try
            {
                // Find header size (first export's SerialOffset)
                long headerSize = asset.Exports.Min(e => e.SerialOffset);
                
                // Calculate this export's start position in .uexp
                long startInUexp = export.SerialOffset - headerSize;
                
                // Find end position by looking at the next export OR end of file
                long endInUexp;
                
                // Sort exports by SerialOffset to find the next one
                var sortedExports = asset.Exports
                    .OrderBy(e => e.SerialOffset)
                    .ToList();
                
                int currentIndex = sortedExports.IndexOf(export);
                
                if (currentIndex < sortedExports.Count - 1)
                {
                    // There's a next export - use its position
                    var nextExport = sortedExports[currentIndex + 1];
                    endInUexp = nextExport.SerialOffset - headerSize;
                }
                else
                {
                    // This is the last export - goes to end of .uexp file
                    string uexpPath = asset.FilePath.Replace(".uasset", ".uexp");
                    if (File.Exists(uexpPath))
                    {
                        endInUexp = new FileInfo(uexpPath).Length;
                    }
                    else
                    {
                        Console.Error.WriteLine($"Warning: .uexp file not found for {asset.FilePath}");
                        return export.SerialSize;
                    }
                }
                
                long calculatedSize = endInUexp - startInUexp;
                
                Console.Error.WriteLine($"[DEBUG] Export {export.ObjectName}:");
                Console.Error.WriteLine($"[DEBUG]   Original SerialSize (header): {export.SerialSize}");
                Console.Error.WriteLine($"[DEBUG]   Calculated size (from .uexp layout): {calculatedSize}");
                Console.Error.WriteLine($"[DEBUG]   Difference: {calculatedSize - export.SerialSize}");
                
                // Only fix if calculated size differs from header
                // This handles cases where the mod's original SerializeSize is incorrect
                if (calculatedSize != export.SerialSize)
                {
                    Console.Error.WriteLine($"[DEBUG]   -> Fixing: {export.SerialSize} → {calculatedSize}");
                    return calculatedSize;
                }
                
                Console.Error.WriteLine($"[DEBUG]   -> No fix needed (sizes match)");
                return export.SerialSize;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Warning: Could not calculate size for export {export.ObjectName}: {ex.Message}");
                return export.SerialSize;
            }
        }

        static int BatchDetectAssets(string[] args)
        {
            if (args.Length < 2)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer batch_detect <directory> [usmap_path]");
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

            foreach (var uassetPath in uassetFiles)
            {
                try
                {
                    // Use UNKNOWN for unversioned files
                    var asset = new UAsset(uassetPath, EngineVersion.UNKNOWN);
                    
                    // Load mappings if provided
                    if (!string.IsNullOrEmpty(usmapPath) && File.Exists(usmapPath))
                    {
                        var mappings = new Usmap(usmapPath);
                        asset.Mappings = mappings;
                    }
                    
                    string assetType = DetermineAssetType(asset);

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

            // Group by asset type
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
    }
}
