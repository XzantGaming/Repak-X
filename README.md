# Repak GUI (Marvel Rivals)

Repak GUI is a Windows application for installing and repacking Marvel Rivals mods. It automates UE IOStore packaging, applies mesh fixes during repack, and intelligently handles compression so your installed mods load and stay small.
 
## Overview

Repak GUI focuses on one job: taking Marvel Rivals `.pak` mods and turning them into game-ready IOStore assets with minimal manual work.

At a high level it:
- Extracts the original `.pak` mod.
- Applies mesh fixes where needed.
- Rebuilds the mod as UE5.3 IOStore (`.utoc` / `.ucas` + a small companion `.pak`).
- Uses Oodle compression intelligently so files stay small but still load correctly.

## Main Components

- **Tauri desktop app (Rust + React GUI)**  
  The main application you run as `repak-gui.exe`. Provides the drag-and-drop interface and orchestrates all work.

- **Rust backend**  
  Handles pak unpacking/packing, IOStore building, and communication with UAsset tools.

- **C# helper tools**  
  - `UAssetBridge.exe` – optional texture processing and conversions (Very experimental at the moment and the patch doesnt properly apply ingame yet).  
  - `StaticMeshSerializeSizeFixer.exe` – mesh fixups applied during repack.

- **UAsset / IOStore logic**  
  Re-uses and extends existing crates to safely read, modify and rebuild UE assets for Marvel Rivals.

End users normally interact only with the GUI; the backend and helper tools run automatically.

## Requirements

- Windows x64
- Marvel Rivals (UE5.3)
- Game-provided Oodle available at runtime (standard for the game)

## Installation

1. Download a prebuilt ZIP from the **Releases** page.
2. Extract it to any writable folder (avoid `Program Files` to keep permissions simple).
3. Run `repak-gui.exe`.

## Basic Usage

1. Drag a `.pak` mod into the GUI.
2. Choose **Repack / Install**.
3. The app will:
   - Unpack the mod
   - Apply mesh fixes
   - Rebuild it as IOStore with appropriate compression
4. The final files are written to your Marvel Rivals `Paks/~mods` folder, typically as:
   - `<stem>_9999999_P.utoc`
   - `<stem>_9999999_P.ucas`
   - `<stem>_9999999_P.pak` (small, uncompressed companion)

Use the `_9999999_P` (The App will autocomplete it for you if its missing but if you want to be extra safe do the suffix manually) suffix so the game prioritizes your mod over base content.

## How It Works (Short Version)

- Rebuilds mods targeting **UE 5.3 IOStore** format.
- Uses **Oodle compression** on data that benefits from it, leaving required headers uncompressed.
- Provides a short compression summary in logs so you can verify that data packed as expected.
- Tries to be robust against malformed or unusual assets to reduce crash risk.

## Troubleshooting

- **Textures look wrong or warnings mention `UAssetBridge.exe`**  
  Add `uassetbridge/UAssetBridge.exe` next to `repak-gui.exe` to enable the full texture pipeline.(Experimental. Being worked on proper function)

- **Mod not loading or game issues**  
  Double-check that the output files are in the correct `Paks/~mods` folder and use the `_9999999_P` suffix.  
  If problems persist, open an issue and include the log file and mod name.

## Development

The project is a Tauri (Rust) + React app with C# helper tools. For building from source, scripts such as `build_contributor.ps1` and `build_app.ps1` orchestrate the full pipeline.

## Acknowledgements

- [unpak](https://github.com/bananaturtlesandwich/unpak): original crate featuring read-only pak operations
- [rust-u4pak](https://github.com/panzi/rust-u4pak)'s README detailing the pak file layout
- [jieyouxu](https://github.com/jieyouxu) for serialization implementation of the significantly more complex V11 index
- [repak](https://github.com/trumank/repak) for the original repak implementation
- [repak-rivals](https://github.com/natimerry/repak-rivals) by @natimerry, the original fork point and an important early reference