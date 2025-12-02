# Technical Documentation & Tool Reference

This document provides in-depth technical details about the internal tools, patchers, and the mod installation pipeline of **Repak GUI Revamped**.

## 1. Installation Pipeline

When a mod is installed, it goes through a complex transformation pipeline to ensure compatibility with *Marvel Rivals*.

### Pipeline Flow
1.  **Input Analysis**:
    -   The source `.pak` is read using the `repak` crate.
    -   AES Key: `0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74`
    -   Content is scanned to auto-detect needed patches (Skeletal Meshes, Textures, etc.).
2.  **Extraction**:
    -   Content is extracted to a temporary directory (`std::env::temp_dir`).
3.  **Patching Phase** (Parallel Execution):
    -   **Stage A: Static Mesh Fixes** (SerializeSize)
    -   **Stage B: Skeletal Mesh Fixes** (Physics/LODs)
    -   **Stage C: Texture Fixes** (Mipmaps)
4.  **Containerization**:
    -   **Audio/Movies**: Repacked into a standard `.pak`.
    -   **Standard Assets**: Converted to IoStore (`.utoc`/`.ucas`) + Companion `.pak`.
5.  **Deployment**:
    -   Files are moved to `<GameDir>/~mods/`.
    -   Priority suffix (`_9999999_P`) is applied.

---

## 2. Internal Tools & Patchers

### A. Static Mesh SerializeSize Fixer
*Location: `src/install_mod/install_mod_logic/iotoc.rs`*

**Problem**: Modded static meshes often have incorrect `SerializeSize` headers compared to the game's unversioned assets, causing the game to crash or refuse to load the mesh.

**Solution**: A hybrid Rust/C# approach.
1.  **Detection**: The system calls an external C# tool (`StaticMeshSerializeSizeFixer.exe`) to analyze the `.uasset`.
    -   Requires a `.usmap` mapping file to understand unversioned properties.
2.  **Calculation**: The C# tool calculates the correct size and returns the old vs. new byte values and offsets via JSON/Stdout.
3.  **Binary Patching**: The Rust backend performs the actual file modification:
    ```rust
    // Locates the 8-byte Int64 size in the binary and replaces it
    uasset_data[offset..offset+8].copy_from_slice(&new_size_bytes);
    ```
    *Why Binary Patching?* Re-serializing the asset with a library often changes the byte layout, which breaks compatibility with the `retoc` IoStore builder. Binary patching preserves the exact structure.

### B. Skeletal Mesh Patcher
*Location: `src/install_mod/install_mod_logic/patch_meshes.rs`*

**Problem**: Custom skeletal meshes often lack correct physics assets or LOD settings compatible with Rivals, leading to "spaghetti" deformation or crashes.

**Solution**:
-   **Target**: Files ending in `.uasset` inside `*Meshes*` directories.
-   **Mechanism**:
    -   Uses the `uasset_mesh_patch_rivals` library.
    -   Reads the `.uasset` (header) and `.uexp` (exports).
    -   Modifies export data to fix buffer sizes and physics asset references.
    -   Rebuilds the `.uexp` file.

### C. Texture Patcher (Mipmap Stripper)
*Location: `src/install_mod/install_mod_logic/iotoc.rs` -> `process_texture_files`*

**Problem**: Custom textures often have mipmaps enabled. In *Marvel Rivals*, streaming logic can cause these to look blurry or fail to load if not properly streamed.

**Solution**:
-   **Target**: `.uasset` files in `*Texture*` directories.
-   **Mechanism**:
    -   Uses `UAssetAPI` (via C# interop) to open the asset.
    -   Locates the `MipGenSettings` property.
    -   Forces it to `NoMipmaps`.
    -   Saves the asset.

### D. IoStore Converter (Retoc)
*Location: `src/install_mod/install_mod_logic/iotoc.rs` -> `convert_to_iostore_directory`*

**Problem**: Modern Unreal Engine games (UE5.3+) use the Zen IoStore system (`.utoc` / `.ucas`) instead of monolithic `.pak` files for faster loading. Standard PAK mods often fail to load or cause stutters.

**Solution**:
-   **Library**: Uses the `retoc` crate (Rust implementation of IoStore).
-   **Process**:
    1.  **UTOC (Table of Contents)**: Generates a header listing all chunks.
    2.  **UCAS (Container Asset Stream)**: Writes the actual compressed data.
    3.  **Companion PAK**: Generates a small `.pak` file containing *only* a `chunknames` text file.
        -   *Why?* The game's mount system looks for `.pak` files to identify mod presence, but reads the actual data from the matching `.utoc`/`.ucas` files.
-   **Compression**: Uses Oodle (via `oodle_loader`) or Zlib depending on configuration.

---

## 3. External Dependencies

| Tool | Purpose | Location |
|------|---------|----------|
| **StaticMeshSerializeSizeFixer** | C# Tool for calculating mesh sizes | `tools/` or relative path |
| **UAssetAPI** | C# Library for reading/writing UE assets | Embedded in C# tools |
| **repak** | Rust Crate for PAK manipulation | `Cargo.toml` |
| **retoc** | Rust Crate for IoStore generation | `Cargo.toml` |
| **oodle_loader** | Rust Crate for Oodle compression | `Cargo.toml` |
| **regex-lite** | Rust Crate for priority suffix parsing | `Cargo.toml` |

## 4. Priority System Technicals

**Logic**: `src/main_tauri.rs` -> `set_mod_priority`

The priority system exploits Unreal Engine's alphabetical loading order.
-   **Format**: `[Name]_[Nines]_P.pak`
-   **Regex**: `r"^(.*)_(\d+)$"`
-   **Behavior**:
    -   `Mod_P.pak` (Default)
    -   `Mod_9999999_P.pak` (7 Nines - High Priority)
    -   `Mod_99999999_P.pak` (8 Nines - Higher Priority)
-   **Conflict Handling**: The installer tracks `mod_type` (e.g., "Spider-Man"). If it sees a second mod of the same type, it increments the requested nines count by 1 before generating the filename.
