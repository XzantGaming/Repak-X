# UAssetEditor
A C# Unreal I/O UAsset Editor for Fortnite

## Features
- Mount I/O containers (.utoc) and read packages compressed with Oodle
- Deserialize unversioned uassets with provided mappings
- Modify uasset data/ properties easily
- Serialize the modified asset back into a file

## How to use
```csharp
// Start logging
Logger.StartLogger();

// Create system
var system = new UnrealFileSystem(@"C:\Program Files\Epic Games\Fortnite\FortniteGame\Content\Paks");

// Add aes keys
system.AesKeys.Add(new FGuid(), new FAesKey("0x0000...")); // Replace with your game's AesKey

// Start a stopwatch
var sw1 = Stopwatch.StartNew();

// Mount containers
system.Initialize();

// Write stats
sw1.Stop();
Console.WriteLine($"Read all in {sw1.ElapsedMilliseconds}ms.");

// Load mappings
system.LoadMappings("path/to/usmap/++Fortnite+Release-33.11-CL-38773622-Windows_oo.usmap", "path/to/dll/oo2core_9_win64.dll (if needed)");

// Initialize Oodle
Oodle.Initialize("oo2core_9_win64.dll");

// Extract the asset
if (!system.TryExtractAsset(
        "FortniteGame/Plugins/GameFeatures/BRCosmetics/Content/Athena/Items/Cosmetics/Characters/CID_028_Athena_Commando_F.uasset",
        out var asset))
    throw new KeyNotFoundException("Unable to find asset.");

// Start a stopwatch
var sw2 = Stopwatch.StartNew();

// Read everything
asset!.ReadAll();

// Write stats
sw2.Stop();
Console.WriteLine($"Read all in {sw2.ElapsedMilliseconds}ms.");

var json = asset.ToString(); // Convert to Json String
File.WriteAllText("CID_028_Athena_Commando_F.json", json);

// Get ItemName Property
var text = asset["CID_028_Athena_Commando_F"]?["ItemName"]?.GetValue<TextProperty>();

if (text?.Value == null)
    throw new NoNullAllowedException("Could not get ItemName property!");

// Set new FText value
text.Value.Text = "Very cool item!";

// Create a writer with the file "CID_028_Athena_Commando_F.uasset"
var writer = new Writer("CID_028_Athena_Commando_F.uasset");

// Serialize the asset and dispose the writer
asset.WriteAll(writer);
writer.Close();

// Create a new ZenAsset with the asset with just serialized
var testAsset = new ZenAsset("CID_028_Athena_Commando_F.uasset");

// Set the GlobalReader instance
var globalToc = system.GetGlobalReader();
testAsset.Initialize(globalToc!);

// Set mappings
testAsset.Mappings = system.Mappings;

// Test if it reads our asset properly
testAsset.ReadAll();
```

## TODOs
- Export Types (Blueprints, etc.)
- Paks and versioned uassets
- Desktop app
- Documentation (wip)
- Engine version switching (only 5.6 works)

### Special thanks to
This project was heavily based on [CUE4Parse](https://github.com/FabianFG/CUE4Parse) with the goal of recreating Unreal Engine's asset reading and writing functionality as an easy to use API
