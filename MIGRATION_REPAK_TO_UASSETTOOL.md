# Migration: Replace `repak/` crate with UAssetTool

## Overview

The `repak/` Rust crate and `oodle_loader/` are now redundant — UAssetTool (C#) handles all PAK and IoStore operations natively. This document tracks what needs to change to fully remove `repak/` from the project.

## What UAssetTool already supports

- **PAK extraction** — `extract_pak` (AES, Oodle/Zlib/Zstd)
- **IoStore creation** — `create_mod_iostore` (replaces PAK write + IoStore conversion)
- **IoStore extraction** — `extract_iostore_legacy`
- **Recompression** — `recompress_iostore`
- **PAK listing** — built into extract_pak with `--list`

## Files that use `repak::`

| File | Usage Count | What it does |
|------|-------------|--------------|
| `main_tauri.rs` | 19 refs | `PakBuilder`, `PakReader`, `AesKey`, `Compression` for parsing dropped files, extracting PAKs, recompression, mod details, clash detection |
| `install_mod.rs` | 7 refs | `PakBuilder`, `PakReader`, `AesKey`, `Compression::Oodle` for reading PAK contents during mod mapping |
| `install_mod_logic/pak_files.rs` | 9 refs | `PakBuilder`, `PakWriter`, `Version`, `repak::Error` for extracting and creating PAK files |
| `install_mod_logic/iotoc.rs` | 3 refs | `repak::Error` as return type for IoStore conversion |

## Migration steps

### 1. Add missing wrappers to `uasset_toolkit/uasset_app/src/lib.rs`

- **`list_pak_files(pak_path, aes_key) -> Vec<String>`** — list files inside a PAK without extracting
- **`extract_pak(pak_path, output_dir, aes_key, filter) -> usize`** — extract PAK to directory
- **`create_pak(input_dir, output_path, compression, aes_key)`** — create a new PAK file (if still needed separately from `create_mod_iostore`)

### 2. Replace `repak::PakBuilder` / `PakReader` usage

Everywhere that does:
```rust
let reader = repak::PakBuilder::new().key(AES_KEY).reader(&mut BufReader::new(file));
let files = reader.files();
```
Replace with UAssetTool calls via `uasset_toolkit`.

### 3. Replace `repak::PakWriter` usage (pak_files.rs)

The `repack_directory_to_pak()` function creates PAK files using `PakWriter`. Replace with `uasset_toolkit::create_mod_iostore()` or a new `create_pak` wrapper.

### 4. Replace `repak::Error` return types

`iotoc.rs` and `pak_files.rs` use `repak::Error` as their return type. Change to `anyhow::Error` or a custom error type.

### 5. Replace `repak::Compression` enum

`InstallableMod` stores `compression: Compression` from repak. Either:
- Remove the field (UAssetTool handles compression internally)
- Replace with a simple string enum

### 6. Replace `repak::utils::AesKey`

The `AES_KEY` static in `install_mod.rs` uses `repak::utils::AesKey`. Pass the hex string directly to UAssetTool instead.

### 7. Update Cargo.toml

- Remove `repak = { path = "../repak", features = ["oodle", "encryption"] }` from `repak-gui/Cargo.toml`
- Remove `"repak"` from workspace members in root `Cargo.toml`
- Remove `"oodle_loader"` reference (it's a dependency of repak)

### 8. Delete directories

- `repak/`
- `oodle_loader/`

## Also remove (confirmed dead)

- `lib/` — old `UAssetEditor.dll` + source, zero references
- `dist-workspace.toml` — unused cargo-dist config
- `release.toml` — unused cargo-release config
- `BUILD_AND_TEST.bat` — old P2P testing script
- `package-lock.json` (root) — orphaned, real one is in `repak-gui/`
