# Unreleased

# Version 2.6.4 (2025-09-29)

## What's new
- Automatic update check on startup.
- "Update available" button appears next to the "💖 Donate" button when a new version is out.
- One‑click update on Windows (downloads and installs the MSI).

## Improvements
- The required texture bridge is now built and packaged automatically, so the texture pipeline works out of the box.
- Added a packaging script that creates a ready‑to‑share folder and ZIP under `dist/`.

## Fixes
- Stability fixes and minor UI polish.

# Version 2.6.3 (2025-09-19)

## Fixes
- Fix color palette editor.

## UI
- Make displayed mod name bold and render the internal suffix (e.g., `_<digits>_P`) transparent.
- added more selection tool options

# Version 2.6.2 (2025-09-19)

## Fixes
- Add restart prompt after changing the Mod folder directory so newly loaded mods appear correctly.
- Persist the selected Mod folder path immediately on change and before exit; restore it at startup so the path shows after restart.

## UX
- Debounce path edits: only commit on field focus loss (to avoid repeated prompts while typing).

# Version 2.6.1 (2025-09-19)

## Changes
- Bug fixes and stability improvements
- More thorough Tag customization across the application

# Version 2.6.0 (2025-09-19)

## Highlights
- Restore and enforce UCAS compression (Oodle), including ExportBundleData; ContainerHeader remains uncompressed.
- Increase IOStore compression block size to 128 KB for better ratios.
- Harden legacy and zen asset paths to avoid index-out-of-bounds panics.
- Add concise compression summary after build.
- Keep companion `chunknames` .pak uncompressed; target UE5_3.

## Changes
- retoc-rivals: IoStore writer now compresses all chunk types except ContainerHeader.
- retoc-rivals: `legacy_asset.rs` name map and package object path resolution are bounds-checked.
- retoc-rivals: `zen_asset_conversion.rs` preload dependency indices are bounds-checked.
- repak-gui: IOStore action set to EngineVersion::UE5_3; companion .pak stays uncompressed.
- Logging: finalize prints `IoStore compression summary: total_blocks_compressed=X bulk=Y shaders=Z export=W`.

# Version 2.5.4 (2025-05-06)

## Changes:
- Fix removal of tempdir causes issues in install mods


# Version 2.5.2 (2025-05-06)

This release contains a window asking users to donate to the project.

## Changes:
- Clean up temporary directories after creating them.

# Version 2.5.1 (2025-05-06)

## New features:
- Ability to install mods from zip files directly
- Show packaging type in install window
- Allow users to unselect specific mods when installing in batch

# Version 2.4.0 (2025-05-05)

This release contains code simplification and bug fixes.

## Changes:
- Added ability to fix dragged .zip/.rar files containing one or more `.pak` files into repak gui 

# Version 2.4.0 (2025-05-04)

This release contains QOL improvements and movie mod fixes for the mod manager.

## Changes:
- Simplify mod type detection
- Add mod type detection for IOStore mods
- Add emma frost skin names to mod categories

## What's broken
- Modtype while importing zip / rar files still doesnt work. This requires extra work

# Version 2.3.0 (2025-05-04)

This release contains QOL improvements and movie mod fixes for the mod manager.

Changes:
 - Removed option to set audio mod manually, this is done automatically for audio mods now.
 - Make movie mods pak the same way as audio mods
 - Remove retrieving filenames from chunkname, instead use the iostore manifest