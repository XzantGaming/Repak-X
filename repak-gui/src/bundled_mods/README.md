# Bundled Mods

This folder contains mods that are bundled with the application and auto-deployed to the game's mods folder.

## LOD Disabler (SK_LODs_Disabler_9999999_P.pak)

The LOD Disabler mod prevents character textures from being reverted to vanilla at far distances.

**Important**: This mod must remain as a legacy .pak file and should NOT be converted to IoStore format.

### How to bundle the mod:

1. Download the Character LODs Disabler from: https://www.nexusmods.com/marvelrivals/mods/5303
2. Place the .pak file in this folder and rename it to `SK_LODs_Disabler_9999999_P.pak`
3. Build with the feature flag enabled:
   ```bash
   cargo build --release --features bundled_lod_mod
   ```

### Build Commands:

**Without bundled mod (default):**
```bash
cargo build --release
```

**With bundled mod:**
```bash
cargo build --release --features bundled_lod_mod
```

### Behavior:

- **With feature enabled**: The mod is embedded in the executable and auto-deployed to `~mods/_LOD-Disabler (Built-in)/` when the app detects a valid game installation.
- **Without feature**: The app compiles normally but won't auto-deploy the LOD mod. Users can still manually install it.

### Folder Structure:
```
~mods/
├── _LOD-Disabler (Built-in)/
│   └── SK_LODs_Disabler_9999999_P.pak    <- Auto-deployed, DO NOT DELETE
├── YourOtherMods/
└── ...
```
