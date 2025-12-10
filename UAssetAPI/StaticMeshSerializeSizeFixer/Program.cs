#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Reflection;
using UAssetAPI;
using UAssetAPI.UnrealTypes;
using UAssetAPI.ExportTypes;
using UAssetAPI.Unversioned;
using UAssetAPI.PropertyTypes.Objects;
using UAssetAPI.PropertyTypes.Structs;

namespace UAssetMeshFixer
{
    class Program
    {
        static int Main(string[] args)
        {
            if (args.Length < 1)
            {
                Console.Error.WriteLine("Usage: UAssetMeshFixer <command> <args>");
                Console.Error.WriteLine("");
                Console.Error.WriteLine("Commands:");
                Console.Error.WriteLine("  detect <uasset_path> [usmap_path]       - Detect asset type (static_mesh, skeletal_mesh, material_instance, or other)");
                Console.Error.WriteLine("  fix <uasset_path> [usmap_path]          - Fix SerializeSize for Static Mesh assets");
                Console.Error.WriteLine("  batch_detect <directory> [usmap_path]   - Detect all .uasset files in directory");
                Console.Error.WriteLine("  dump <uasset_path> <usmap_path>         - Dump detailed info about asset structure (for analysis)");
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
                    "dump" => DumpAssetInfo(args),
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

        static int DumpAssetInfo(string[] args)
        {
            if (args.Length < 3)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer dump <uasset_path> <usmap_path>");
                return 1;
            }

            string uassetPath = args[1];
            string usmapPath = args[2];

            if (!File.Exists(uassetPath))
            {
                Console.Error.WriteLine($"File not found: {uassetPath}");
                return 1;
            }

            if (!File.Exists(usmapPath))
            {
                Console.Error.WriteLine($"USmap file not found: {usmapPath}");
                return 1;
            }

            try
            {
                Console.WriteLine("=== Asset Dump ===");
                Console.WriteLine($"File: {uassetPath}");
                Console.WriteLine();

                // Load mappings
                var mappings = new Usmap(usmapPath);
                Console.WriteLine($"[INFO] Loaded USmap: {usmapPath}");

                // Load asset
                var asset = new UAsset(uassetPath, EngineVersion.VER_UE5_3);
                asset.Mappings = mappings;
                asset.UseSeparateBulkDataFiles = true;

                Console.WriteLine($"[INFO] Asset loaded successfully");
                Console.WriteLine();

                // Basic info
                Console.WriteLine("=== Basic Info ===");
                Console.WriteLine($"Engine Version: {asset.GetEngineVersion()}");
                Console.WriteLine($"Export Count: {asset.Exports.Count}");
                Console.WriteLine($"Import Count: {asset.Imports.Count}");
                Console.WriteLine();

                // Dump imports
                Console.WriteLine("=== Imports ===");
                for (int i = 0; i < asset.Imports.Count; i++)
                {
                    var import = asset.Imports[i];
                    Console.WriteLine($"  [{i}] {import.ObjectName?.Value?.Value} (Class: {import.ClassName?.Value?.Value}, Package: {import.ClassPackage?.Value?.Value})");
                }
                Console.WriteLine();

                // Dump exports
                Console.WriteLine("=== Exports ===");
                for (int i = 0; i < asset.Exports.Count; i++)
                {
                    var export = asset.Exports[i];
                    string className = "Unknown";
                    if (export.ClassIndex.IsImport())
                    {
                        var classImport = export.ClassIndex.ToImport(asset);
                        className = classImport?.ObjectName?.Value?.Value ?? "Unknown";
                    }

                    Console.WriteLine($"  [{i}] {export.ObjectName?.Value?.Value}");
                    Console.WriteLine($"      Class: {className}");
                    Console.WriteLine($"      SerialOffset: 0x{export.SerialOffset:X}");
                    Console.WriteLine($"      SerialSize: {export.SerialSize} bytes");
                    Console.WriteLine($"      Export Type: {export.GetType().Name}");

                    // If it's a NormalExport, dump properties
                    if (export is NormalExport normalExport)
                    {
                        Console.WriteLine($"      Properties: {normalExport.Data.Count}");
                        foreach (var prop in normalExport.Data)
                        {
                            DumpProperty(prop, 8);
                        }

                        // Check for extra data (raw binary data after properties)
                        if (normalExport.Extras != null && normalExport.Extras.Length > 0)
                        {
                            Console.WriteLine($"      Extra Data: {normalExport.Extras.Length} bytes");
                            
                            // For skeletal meshes, try to analyze the extra data
                            if (className == "SkeletalMesh")
                            {
                                AnalyzeSkeletalMeshExtras(normalExport.Extras);
                            }
                        }
                    }
                    Console.WriteLine();
                }

                // Check usmap for relevant struct definitions
                Console.WriteLine("=== USmap Struct Definitions (Skeletal Mesh Related) ===");
                DumpRelevantUsmapStructs(mappings);

                return 0;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"Error: {ex.Message}");
                Console.Error.WriteLine(ex.StackTrace);
                return 1;
            }
        }

        static void DumpProperty(PropertyData prop, int indent)
        {
            string indentStr = new string(' ', indent);
            string propName = prop.Name?.Value?.Value ?? "<unnamed>";
            string propType = prop.PropertyType?.ToString() ?? prop.GetType().Name;

            if (prop is ArrayPropertyData arrayProp)
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType}): [{arrayProp.Value?.Length ?? 0} items]");
                if (arrayProp.Value != null && arrayProp.Value.Length > 0 && arrayProp.Value.Length <= 10)
                {
                    foreach (var item in arrayProp.Value)
                    {
                        DumpProperty(item, indent + 4);
                    }
                }
                else if (arrayProp.Value != null && arrayProp.Value.Length > 10)
                {
                    Console.WriteLine($"{indentStr}    (showing first 3 items)");
                    for (int j = 0; j < Math.Min(3, arrayProp.Value.Length); j++)
                    {
                        DumpProperty(arrayProp.Value[j], indent + 4);
                    }
                }
            }
            else if (prop is StructPropertyData structProp)
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType}): {structProp.StructType?.Value?.Value ?? "?"}");
                if (structProp.Value != null && structProp.Value.Count <= 10)
                {
                    foreach (var subProp in structProp.Value)
                    {
                        DumpProperty(subProp, indent + 4);
                    }
                }
            }
            else if (prop is ObjectPropertyData objProp)
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType}): Index={objProp.Value.Index}");
            }
            else if (prop is NamePropertyData nameProp)
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType}): {nameProp.Value?.Value?.Value ?? "null"}");
            }
            else if (prop is IntPropertyData intProp)
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType}): {intProp.Value}");
            }
            else if (prop is BoolPropertyData boolProp)
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType}): {boolProp.Value}");
            }
            else
            {
                Console.WriteLine($"{indentStr}- {propName} ({propType})");
            }
        }

        static void AnalyzeSkeletalMeshExtras(byte[] extras)
        {
            Console.WriteLine($"      --- Skeletal Mesh Extra Data Analysis ---");
            Console.WriteLine($"      Total extra bytes: {extras.Length}");

            // The Rust patcher searches for 0xff 0xff 0xff followed by NOT 0xff
            // Then reads material_count from 8 bytes BEFORE the pattern (actually 4 bytes before the 0xff 0xff 0xff)
            // Let's search more broadly
            List<(int offset, int count)> candidates = new List<(int, int)>();
            
            for (int i = 4; i < extras.Length - 4; i++)
            {
                if (extras[i] == 0xff && extras[i + 1] == 0xff && extras[i + 2] == 0xff)
                {
                    // Check if next byte is NOT 0xff (like the Rust patcher does)
                    if (extras[i + 3] != 0xff)
                    {
                        // Rust code: r.seek_relative(-8)? then reads material_count
                        // That means: from position after reading 3 bytes (0xff 0xff 0xff), go back 8 bytes
                        // So material_count is at position: i - 5 (i is start of 0xff 0xff 0xff)
                        // Actually: after reading [0xff, 0xff, 0xff], position is i+3
                        // seek_relative(-8) goes to i+3-8 = i-5
                        // Then read_i32 reads 4 bytes at i-5
                        if (i >= 5)
                        {
                            int potentialCount = BitConverter.ToInt32(extras, i - 5);
                            if (potentialCount > 0 && potentialCount < 255)
                            {
                                candidates.Add((i, potentialCount));
                                Console.WriteLine($"      Found pattern at offset 0x{i:X}: 0xff 0xff 0xff {extras[i+3]:X2}, material_count (at 0x{i-5:X}) = {potentialCount}");
                            }
                        }
                    }
                }
            }

            if (candidates.Count == 0)
            {
                Console.WriteLine($"      No material pattern found using Rust patcher logic");
                
                // Let's also search for any reasonable material counts followed by 40-byte structures
                Console.WriteLine($"      Searching for alternative patterns...");
                for (int i = 0; i < Math.Min(extras.Length - 200, 500000); i++)
                {
                    int count = BitConverter.ToInt32(extras, i);
                    if (count > 0 && count <= 30) // Reasonable material count
                    {
                        // Check if there's a pattern of 40-byte entries after this
                        // Each entry might have an object reference (negative index like 0xffffffxx)
                        bool looksLikeMaterials = true;
                        for (int m = 0; m < Math.Min(count, 3); m++)
                        {
                            int entryStart = i + 4 + (m * 40);
                            if (entryStart + 40 > extras.Length)
                            {
                                looksLikeMaterials = false;
                                break;
                            }
                            // First 4 bytes of material entry should be a negative import index
                            int importIdx = BitConverter.ToInt32(extras, entryStart);
                            if (importIdx >= 0 || importIdx < -1000)
                            {
                                looksLikeMaterials = false;
                                break;
                            }
                        }
                        if (looksLikeMaterials && count >= 5)
                        {
                            Console.WriteLine($"      Potential material array at 0x{i:X}: count={count}");
                            // Show first entry
                            Console.WriteLine($"        First entry bytes: {BitConverter.ToString(extras, i + 4, Math.Min(40, extras.Length - i - 4))}");
                        }
                    }
                }
            }
            else
            {
                Console.WriteLine($"      Found {candidates.Count} potential material pattern(s)");
            }

            // Show first 512 bytes as hex dump
            Console.WriteLine($"      First 512 bytes (hex):");
            int bytesToShow = Math.Min(512, extras.Length);
            for (int i = 0; i < bytesToShow; i += 16)
            {
                string hex = "";
                string ascii = "";
                for (int j = 0; j < 16 && i + j < bytesToShow; j++)
                {
                    byte b = extras[i + j];
                    hex += $"{b:X2} ";
                    ascii += (b >= 32 && b < 127) ? (char)b : '.';
                }
                Console.WriteLine($"      {i:X4}: {hex,-48} {ascii}");
            }
        }

        static void DumpRelevantUsmapStructs(Usmap mappings)
        {
            string[] relevantNames = new[]
            {
                "FSkelMeshSection", "FSkelMeshRenderSection", "FSkeletalMaterial",
                "FSkeletalMeshLODInfo", "FSkeletalMeshLODRenderData", "FSkeletalMeshRenderData",
                "FSkelMeshSourceSectionUserData", "FMeshSectionInfo"
            };

            foreach (var kvp in mappings.Schemas)
            {
                string schemaName = kvp.Key;
                var schema = kvp.Value;
                
                if (relevantNames.Any(n => schemaName.Equals(n, StringComparison.OrdinalIgnoreCase)) ||
                    schemaName.Contains("SkelMeshSection", StringComparison.OrdinalIgnoreCase) ||
                    schemaName.Equals("SkeletalMaterial", StringComparison.OrdinalIgnoreCase) ||
                    schemaName.Equals("FSkeletalMaterial", StringComparison.OrdinalIgnoreCase))
                {
                    Console.WriteLine($"  {schemaName} (SuperType: {schema.SuperType ?? "none"})");
                    if (schema.Properties != null)
                    {
                        foreach (var propKvp in schema.Properties)
                        {
                            var prop = propKvp.Value;
                            Console.WriteLine($"    - {prop.Name}: {prop.PropertyData?.Type} (SchemaIdx: {prop.SchemaIndex}, ArrayDim: {prop.ArraySize})");
                        }
                    }
                    Console.WriteLine();
                }
            }
        }

        static int FixSkeletalMeshMaterials(string[] args)
        {
            if (args.Length < 3)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer fix_skel <uasset_path> <usmap_path>");
                return 1;
            }

            string uassetPath = args[1];
            string usmapPath = args[2];

            if (!File.Exists(uassetPath))
            {
                Console.Error.WriteLine($"File not found: {uassetPath}");
                return 1;
            }

            string uexpPath = uassetPath.Replace(".uasset", ".uexp");
            if (!File.Exists(uexpPath))
            {
                Console.Error.WriteLine($"UEXP file not found: {uexpPath}");
                return 1;
            }

            try
            {
                Console.Error.WriteLine($"[INFO] Processing: {uassetPath}");

                // Load mappings
                Usmap? mappings = null;
                if (File.Exists(usmapPath))
                {
                    mappings = new Usmap(usmapPath);
                    Console.Error.WriteLine($"[INFO] Loaded USmap: {usmapPath}");
                }

                // Load asset to verify it's a skeletal mesh and get export info
                var asset = new UAsset(uassetPath, EngineVersion.VER_UE5_3);
                if (mappings != null) asset.Mappings = mappings;
                asset.UseSeparateBulkDataFiles = true;

                // Verify it's a skeletal mesh
                string assetType = DetermineAssetType(asset);
                if (assetType != "skeletal_mesh")
                {
                    Console.Error.WriteLine($"[ERROR] Asset is not a skeletal mesh (detected: {assetType})");
                    return 1;
                }

                // Find the skeletal mesh export
                Export? skelMeshExport = null;
                int skelMeshExportIndex = -1;
                for (int i = 0; i < asset.Exports.Count; i++)
                {
                    var export = asset.Exports[i];
                    if (export.ClassIndex.IsImport())
                    {
                        var classImport = export.ClassIndex.ToImport(asset);
                        if (classImport?.ObjectName?.Value?.Value == "SkeletalMesh")
                        {
                            skelMeshExport = export;
                            skelMeshExportIndex = i;
                            break;
                        }
                    }
                }

                if (skelMeshExport == null)
                {
                    Console.Error.WriteLine($"[ERROR] Could not find SkeletalMesh export");
                    return 1;
                }

                Console.Error.WriteLine($"[INFO] Found SkeletalMesh export at index {skelMeshExportIndex}");
                Console.Error.WriteLine($"[INFO] SerialOffset: 0x{skelMeshExport.SerialOffset:X}, SerialSize: {skelMeshExport.SerialSize}");

                // Read the .uexp file
                byte[] uexpData = File.ReadAllBytes(uexpPath);
                Console.Error.WriteLine($"[INFO] UEXP file size: {uexpData.Length} bytes");

                // Find the material pattern in the uexp
                // The pattern is: [count:4] [negative_fname:4 ending in 0xFF 0xFF 0xFF] [non-0xFF byte]
                // Material data starts right after the pattern
                
                int materialCount = 0;
                int materialDataStart = -1;
                
                for (int i = 4; i < uexpData.Length - 4; i++)
                {
                    if (uexpData[i] == 0xff && uexpData[i + 1] == 0xff && uexpData[i + 2] == 0xff)
                    {
                        if (uexpData[i + 3] != 0xff)
                        {
                            // Check material count at i - 5 (Rust patcher logic)
                            if (i >= 5)
                            {
                                int potentialCount = BitConverter.ToInt32(uexpData, i - 5);
                                if (potentialCount > 0 && potentialCount < 255)
                                {
                                    materialCount = potentialCount;
                                    materialDataStart = i + 4; // After the 0xFF 0xFF 0xFF XX pattern
                                    Console.Error.WriteLine($"[INFO] Found material pattern at offset 0x{i:X}");
                                    Console.Error.WriteLine($"[INFO] Material count: {materialCount}");
                                    Console.Error.WriteLine($"[INFO] Material data starts at: 0x{materialDataStart:X}");
                                    break;
                                }
                            }
                        }
                    }
                }

                if (materialCount == 0 || materialDataStart < 0)
                {
                    Console.Error.WriteLine($"[ERROR] Could not find material pattern in UEXP");
                    return 1;
                }

                // IMPROVEMENT 1: Validate material count against actual material imports
                int actualMaterialImports = asset.Imports.Count(i => 
                    i.ClassName?.Value?.Value == "Material" || 
                    i.ClassName?.Value?.Value == "MaterialInstanceConstant");
                
                if (materialCount != actualMaterialImports)
                {
                    Console.Error.WriteLine($"[WARNING] Detected material count ({materialCount}) differs from actual material imports ({actualMaterialImports})");
                    // Continue anyway - the pattern-based count is what the render data uses
                }
                else
                {
                    Console.Error.WriteLine($"[INFO] Material count validated: {materialCount} matches import count");
                }

                // Note: Already-patched detection is not needed because:
                // 1. Files come fresh from pak extraction in a temp directory
                // 2. The temp directory is cleaned up after each mod installation
                // 3. Double-patching would only happen if someone manually runs this tool twice
                
                Console.Error.WriteLine($"[DEBUG] Header SerialSize: {skelMeshExport.SerialSize}");
                Console.Error.WriteLine($"[DEBUG] UEXP file size: {uexpData.Length}");

                // Create the patched uexp
                // For each material (40 bytes), we add 4 null bytes after it
                int bytesToAdd = materialCount * 4;
                byte[] patchedUexp = new byte[uexpData.Length + bytesToAdd];

                // Copy data before material entries
                Array.Copy(uexpData, 0, patchedUexp, 0, materialDataStart);

                // Copy each material entry (40 bytes) and add 4 null bytes
                int srcOffset = materialDataStart;
                int dstOffset = materialDataStart;
                for (int m = 0; m < materialCount; m++)
                {
                    // Copy 40 bytes of material entry
                    Array.Copy(uexpData, srcOffset, patchedUexp, dstOffset, 40);
                    srcOffset += 40;
                    dstOffset += 40;
                    
                    // Add 4 null bytes
                    patchedUexp[dstOffset++] = 0;
                    patchedUexp[dstOffset++] = 0;
                    patchedUexp[dstOffset++] = 0;
                    patchedUexp[dstOffset++] = 0;
                }

                // Copy remaining data
                int remainingBytes = uexpData.Length - srcOffset;
                Array.Copy(uexpData, srcOffset, patchedUexp, dstOffset, remainingBytes);

                Console.Error.WriteLine($"[INFO] Patched UEXP size: {patchedUexp.Length} bytes (+{bytesToAdd} bytes)");

                // Write the patched uexp
                File.WriteAllBytes(uexpPath, patchedUexp);
                Console.Error.WriteLine($"[INFO] Wrote patched UEXP: {uexpPath}");

                // Now update the uasset header using UAssetAPI's parsed values
                // We need to update:
                // 1. SerialSize of the skeletal mesh export
                // 2. BulkDataStartOffset
                
                long originalSerialSize = skelMeshExport.SerialSize;
                long newSerialSize = originalSerialSize + bytesToAdd;
                
                // Get BulkDataStartOffset using reflection (it's internal)
                var bulkDataField = typeof(UAsset).GetField("BulkDataStartOffset", BindingFlags.NonPublic | BindingFlags.Instance);
                long originalBulkDataStartOffset = bulkDataField != null ? (long)bulkDataField.GetValue(asset)! : 0;
                long newBulkDataStartOffset = originalBulkDataStartOffset + bytesToAdd;
                
                Console.Error.WriteLine($"[INFO] Updating SerialSize: {originalSerialSize} -> {newSerialSize}");
                Console.Error.WriteLine($"[INFO] Updating BulkDataStartOffset: {originalBulkDataStartOffset} -> {newBulkDataStartOffset}");
                
                // Update the values in the asset object
                skelMeshExport.SerialSize = newSerialSize;
                if (bulkDataField != null)
                {
                    bulkDataField.SetValue(asset, newBulkDataStartOffset);
                }
                
                // Now we need to write just the .uasset header, not the .uexp
                // UAssetAPI's Write() would regenerate the .uexp, so we use a workaround:
                // We'll write to memory and only take the .uasset portion
                
                // Temporarily disable separate bulk data files to get just the header
                bool originalUseSeparate = asset.UseSeparateBulkDataFiles;
                
                try
                {
                    // Write the asset - this will update the header with new SerialSize and BulkDataStartOffset
                    asset.Write(out MemoryStream uassetStream, out MemoryStream uexpStream);
                    
                    // Write only the .uasset file (the header)
                    File.WriteAllBytes(uassetPath, uassetStream.ToArray());
                    Console.Error.WriteLine($"[INFO] Wrote patched UASSET: {uassetPath}");
                    
                    // Note: We intentionally DON'T write the uexpStream because we already wrote our patched uexp
                }
                finally
                {
                    asset.UseSeparateBulkDataFiles = originalUseSeparate;
                }

                // Output result as JSON
                var result = new
                {
                    success = true,
                    uasset_path = uassetPath,
                    uexp_path = uexpPath,
                    material_count = materialCount,
                    bytes_added = bytesToAdd,
                    original_serial_size = originalSerialSize,
                    new_serial_size = newSerialSize
                };

                Console.WriteLine(JsonSerializer.Serialize(result, new JsonSerializerOptions { WriteIndented = true }));
                return 0;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine($"[ERROR] {ex.Message}");
                Console.Error.WriteLine(ex.StackTrace);
                return 1;
            }
        }

        // Cache for usmap to avoid reloading
        private static Usmap? _cachedMappings = null;
        private static string? _cachedUsmapPath = null;
        private static DateTime _cachedUsmapModTime = DateTime.MinValue;
        private static readonly object _usmapLock = new object();

        /// <summary>
        /// Get or load usmap with caching. Automatically invalidates cache if file changed.
        /// </summary>
        private static Usmap GetOrLoadUsmap(string usmapPath)
        {
            var fileInfo = new FileInfo(usmapPath);
            
            lock (_usmapLock)
            {
                // Check if cache is valid
                if (_cachedMappings != null && 
                    _cachedUsmapPath == usmapPath && 
                    _cachedUsmapModTime == fileInfo.LastWriteTimeUtc)
                {
                    return _cachedMappings;
                }
                
                // Load and cache
                Console.Error.WriteLine($"[INFO] Loading usmap (cache miss or file changed)...");
                var sw = System.Diagnostics.Stopwatch.StartNew();
                _cachedMappings = new Usmap(usmapPath);
                _cachedUsmapPath = usmapPath;
                _cachedUsmapModTime = fileInfo.LastWriteTimeUtc;
                sw.Stop();
                Console.Error.WriteLine($"[INFO] Usmap loaded in {sw.ElapsedMilliseconds}ms");
                
                return _cachedMappings;
            }
        }

        // Cache reflection field lookup
        private static readonly FieldInfo? _bulkDataField = typeof(UAsset).GetField("BulkDataStartOffset", BindingFlags.NonPublic | BindingFlags.Instance);

        /// <summary>
        /// Optimized batch processing for skeletal meshes.
        /// Key optimizations:
        /// 1. Load usmap ONCE with caching (survives across calls, invalidates on file change)
        /// 2. Parallel processing with configurable thread count
        /// 3. Skip non-skeletal mesh files early using filename heuristics
        /// 4. Use SkipParsingExports flag - only parse header, not export data
        /// 5. Use SkipPreloadDependencyLoading for thread safety
        /// 6. Binary header patching instead of asset.Write()
        /// 7. Cached reflection field lookup
        /// </summary>
        static int BatchFixSkeletalMeshes(string[] args)
        {
            if (args.Length < 3)
            {
                Console.Error.WriteLine("Usage: StaticMeshSerializeSizeFixer batch_fix_skel <directory> <usmap_path> [thread_count]");
                return 1;
            }

            string directory = args[1];
            string usmapPath = args[2];
            int threadCount = args.Length > 3 && int.TryParse(args[3], out int tc) ? tc : Environment.ProcessorCount;

            if (!Directory.Exists(directory))
            {
                Console.Error.WriteLine($"Directory not found: {directory}");
                return 1;
            }

            if (!File.Exists(usmapPath))
            {
                Console.Error.WriteLine($"Usmap file not found: {usmapPath}");
                return 1;
            }

            var sw = System.Diagnostics.Stopwatch.StartNew();

            // OPTIMIZATION 1: Cached usmap loading
            Usmap mappings = GetOrLoadUsmap(usmapPath);

            // OPTIMIZATION 2: Pre-filter by filename to skip obvious non-meshes
            var allUassets = Directory.GetFiles(directory, "*.uasset", SearchOption.AllDirectories);
            var potentialSkeletalMeshes = allUassets
                .Where(f => {
                    string name = Path.GetFileName(f).ToLower();
                    string dir = Path.GetDirectoryName(f)?.ToLower() ?? "";
                    return name.StartsWith("sk_") || dir.Contains("meshes");
                })
                .ToArray();

            Console.Error.WriteLine($"[INFO] Found {allUassets.Length} .uasset files, {potentialSkeletalMeshes.Length} potential skeletal meshes");
            Console.Error.WriteLine($"[INFO] Using {threadCount} threads");

            var results = new System.Collections.Concurrent.ConcurrentBag<object>();
            int processed = 0;
            int patched = 0;
            int skipped = 0;
            int errors = 0;

            // OPTIMIZATION 3: Parallel processing with configurable thread count
            System.Threading.Tasks.Parallel.ForEach(potentialSkeletalMeshes, 
                new System.Threading.Tasks.ParallelOptions { MaxDegreeOfParallelism = threadCount },
                uassetPath =>
            {
                try
                {
                    string uexpPath = uassetPath.Replace(".uasset", ".uexp");
                    if (!File.Exists(uexpPath))
                    {
                        System.Threading.Interlocked.Increment(ref skipped);
                        return;
                    }

                    // OPTIMIZATION 4 & 5: Skip parsing exports and preload dependencies
                    // This dramatically speeds up asset loading - we only need header info
                    var asset = new UAsset(uassetPath, EngineVersion.VER_UE5_3, mappings,
                        CustomSerializationFlags.SkipParsingExports | CustomSerializationFlags.SkipPreloadDependencyLoading);
                    asset.UseSeparateBulkDataFiles = true;

                    // Check if it's actually a skeletal mesh (uses imports, not export data)
                    string assetType = DetermineAssetType(asset);
                    if (assetType != "skeletal_mesh")
                    {
                        System.Threading.Interlocked.Increment(ref skipped);
                        return;
                    }

                    // Find skeletal mesh export
                    Export? skelMeshExport = null;
                    for (int i = 0; i < asset.Exports.Count; i++)
                    {
                        var export = asset.Exports[i];
                        if (export.ClassIndex.IsImport())
                        {
                            var classImport = export.ClassIndex.ToImport(asset);
                            if (classImport?.ObjectName?.Value?.Value == "SkeletalMesh")
                            {
                                skelMeshExport = export;
                                break;
                            }
                        }
                    }

                    if (skelMeshExport == null)
                    {
                        System.Threading.Interlocked.Increment(ref skipped);
                        return;
                    }

                    // Read uexp and find material pattern
                    byte[] uexpData = File.ReadAllBytes(uexpPath);
                    
                    int materialCount = 0;
                    int materialDataStart = -1;
                    
                    // OPTIMIZATION: Search with early termination
                    int searchLimit = Math.Min(uexpData.Length - 4, 500000); // Don't search entire file
                    for (int i = 4; i < searchLimit; i++)
                    {
                        if (uexpData[i] == 0xff && uexpData[i + 1] == 0xff && uexpData[i + 2] == 0xff)
                        {
                            if (uexpData[i + 3] != 0xff && i >= 5)
                            {
                                int potentialCount = BitConverter.ToInt32(uexpData, i - 5);
                                if (potentialCount > 0 && potentialCount < 255)
                                {
                                    materialCount = potentialCount;
                                    materialDataStart = i + 4;
                                    break;
                                }
                            }
                        }
                    }

                    if (materialCount == 0 || materialDataStart < 0)
                    {
                        System.Threading.Interlocked.Increment(ref skipped);
                        return;
                    }

                    // Patch the uexp
                    int bytesToAdd = materialCount * 4;
                    byte[] patchedUexp = new byte[uexpData.Length + bytesToAdd];
                    
                    // Copy before materials
                    Buffer.BlockCopy(uexpData, 0, patchedUexp, 0, materialDataStart);

                    // Copy materials with padding
                    int srcOffset = materialDataStart;
                    int dstOffset = materialDataStart;
                    for (int m = 0; m < materialCount; m++)
                    {
                        Buffer.BlockCopy(uexpData, srcOffset, patchedUexp, dstOffset, 40);
                        srcOffset += 40;
                        dstOffset += 44; // 40 + 4 null bytes (already zeroed in new array)
                    }

                    // Copy remaining
                    Buffer.BlockCopy(uexpData, srcOffset, patchedUexp, dstOffset, uexpData.Length - srcOffset);

                    File.WriteAllBytes(uexpPath, patchedUexp);

                    // OPTIMIZATION 6: Fast binary header patching
                    long originalSerialSize = skelMeshExport.SerialSize;
                    long newSerialSize = originalSerialSize + bytesToAdd;
                    
                    // OPTIMIZATION 7: Cached reflection
                    long originalBulkDataStartOffset = _bulkDataField != null ? (long)_bulkDataField.GetValue(asset)! : 0;
                    long newBulkDataStartOffset = originalBulkDataStartOffset + bytesToAdd;
                    
                    byte[] uassetData = File.ReadAllBytes(uassetPath);
                    
                    // Patch SerialSize
                    byte[] origSizeBytes = BitConverter.GetBytes(originalSerialSize);
                    byte[] newSizeBytes = BitConverter.GetBytes(newSerialSize);
                    byte[] serialOffsetBytes = BitConverter.GetBytes(skelMeshExport.SerialOffset);
                    
                    for (int i = 0; i < uassetData.Length - 16; i++)
                    {
                        if (uassetData[i] == origSizeBytes[0] && 
                            uassetData[i+1] == origSizeBytes[1] &&
                            uassetData[i+2] == origSizeBytes[2] &&
                            uassetData[i+3] == origSizeBytes[3] &&
                            uassetData[i+4] == origSizeBytes[4] &&
                            uassetData[i+5] == origSizeBytes[5] &&
                            uassetData[i+6] == origSizeBytes[6] &&
                            uassetData[i+7] == origSizeBytes[7])
                        {
                            // Verify SerialOffset follows
                            if (uassetData[i+8] == serialOffsetBytes[0] &&
                                uassetData[i+9] == serialOffsetBytes[1] &&
                                uassetData[i+10] == serialOffsetBytes[2] &&
                                uassetData[i+11] == serialOffsetBytes[3] &&
                                uassetData[i+12] == serialOffsetBytes[4] &&
                                uassetData[i+13] == serialOffsetBytes[5] &&
                                uassetData[i+14] == serialOffsetBytes[6] &&
                                uassetData[i+15] == serialOffsetBytes[7])
                            {
                                Buffer.BlockCopy(newSizeBytes, 0, uassetData, i, 8);
                                break;
                            }
                        }
                    }
                    
                    // Patch BulkDataStartOffset
                    if (originalBulkDataStartOffset > 0)
                    {
                        byte[] origBulkBytes = BitConverter.GetBytes(originalBulkDataStartOffset);
                        byte[] newBulkBytes = BitConverter.GetBytes(newBulkDataStartOffset);
                        
                        for (int i = 0; i < uassetData.Length - 8; i++)
                        {
                            if (uassetData[i] == origBulkBytes[0] &&
                                uassetData[i+1] == origBulkBytes[1] &&
                                uassetData[i+2] == origBulkBytes[2] &&
                                uassetData[i+3] == origBulkBytes[3] &&
                                uassetData[i+4] == origBulkBytes[4] &&
                                uassetData[i+5] == origBulkBytes[5] &&
                                uassetData[i+6] == origBulkBytes[6] &&
                                uassetData[i+7] == origBulkBytes[7])
                            {
                                Buffer.BlockCopy(newBulkBytes, 0, uassetData, i, 8);
                                break;
                            }
                        }
                    }
                    
                    File.WriteAllBytes(uassetPath, uassetData);

                    results.Add(new
                    {
                        path = uassetPath,
                        material_count = materialCount,
                        bytes_added = bytesToAdd
                    });

                    System.Threading.Interlocked.Increment(ref patched);
                }
                catch (Exception ex)
                {
                    Console.Error.WriteLine($"[ERROR] {Path.GetFileName(uassetPath)}: {ex.Message}");
                    System.Threading.Interlocked.Increment(ref errors);
                }
                finally
                {
                    System.Threading.Interlocked.Increment(ref processed);
                }
            });

            sw.Stop();

            var summary = new
            {
                success = true,
                total_files = allUassets.Length,
                potential_skeletal_meshes = potentialSkeletalMeshes.Length,
                processed = processed,
                patched = patched,
                skipped = skipped,
                errors = errors,
                elapsed_ms = sw.ElapsedMilliseconds,
                patched_files = results.ToArray()
            };

            Console.WriteLine(JsonSerializer.Serialize(summary, new JsonSerializerOptions { WriteIndented = true }));
            Console.Error.WriteLine($"[INFO] Completed in {sw.ElapsedMilliseconds}ms ({patched} patched, {skipped} skipped, {errors} errors)");

            return errors > 0 ? 1 : 0;
        }
    }
}
