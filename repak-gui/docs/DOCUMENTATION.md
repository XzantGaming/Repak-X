# Repak GUI Revamped - Tauri Update Documentation

## Overview
**Repak GUI Revamped (Tauri Update)** is a modern mod manager for *Marvel Rivals*, replacing the previous immediate-mode GUI (egui) with a robust web-based frontend (React) and a high-performance Rust backend (Tauri). This architecture allows for a more responsive UI, better theming, and seamless integration with the file system.

## Project Structure
The project is located in `Repak_Gui-Revamped-TauriUpdate/`.

```
Repak_Gui-Revamped-TauriUpdate/
├── repak-gui/                  # Main Application Root
│   ├── src/                    # Source Code (Shared Rust & Frontend)
│   │   ├── components/         # React UI Components
│   │   ├── install_mod/        # Core Mod Installation Logic (Rust)
│   │   ├── install_mod.rs      # Mod Struct Definitions
│   │   ├── main_tauri.rs       # Backend Entry Point & Commands
│   │   ├── App.jsx             # Main Frontend Application
│   │   └── ...
│   ├── package.json            # Frontend Dependencies (Vite, React, Tauri API)
│   ├── Cargo.toml              # Backend Dependencies (Rust)
│   └── tauri.conf.json         # Tauri Configuration
├── dist/                       # Compiled Frontend Assets
├── build_project.bat           # One-click Build Script
└── run_dev.bat                 # Development Launch Script
```

## Key Features

### 1. Intelligent Mod Installation
- **Drag & Drop**: Users can drag `.pak` files directly into the window.
- **Auto-Detection**: The backend analyzes the PAK file content to detect:
    - **Skeletal Meshes**: Auto-enables "Fix Mesh" (physics/LOD fixes).
    - **Textures**: Auto-enables "Fix Textures" (mip settings).
    - **Static Meshes**: Auto-enables "Fix SerializeSize".
- **Repak Engine**: Converts standard PAK files into the game's native **IoStore** format (`.utoc`, `.ucas`, `.pak`) for compatibility.

### 2. Dynamic Priority System
The game loads mods based on filename alphabetical order. This project implements a "Nines" suffix system to ensure mods override default files.
- **Suffix Format**: `_9999999_P` (minimum 7 nines).
- **Conflict Resolution**: When installing multiple mods of the same type (e.g., "Spider-Man"), the system automatically increments the priority (7 nines, 8 nines, etc.) to prevent conflicts.
- **Manual Control**: Users can edit the priority number directly in the UI, which renames the files on disk instantly.

### 3. State Management & Persistence
- **Backend State**: `AppState` (in `main_tauri.rs`) holds the game path, mod folder organization, and metadata.
- **Persistence**: Data is saved to `app_state.json` (or similar config in the app data directory) to remember folders and custom names between sessions.

### 4. File Watching
- **Real-time**: Uses `notify` to watch the `~mods` directory.
- **Sync**: Any external change (adding/deleting a file) immediately triggers a UI refresh via the `mods_dir_changed` event.

## Core Components

### Backend (Rust)
**`src/main_tauri.rs`**
- The nervous system of the app.
- Registers Tauri commands: `install_mods`, `get_pak_files`, `set_mod_priority`, `start_file_watcher`, etc.
- Manages the `install_mod_logic` integration.

**`src/install_mod/install_mod_logic.rs`**
- **`normalize_mod_base_name`**: Handles the priority suffix logic using Regex.
- **`install_mods_in_viewport`**: The main loop that processes a queue of mods, performing patches and copying files to the game directory.

### Frontend (React)
**`src/App.jsx`**
- Manages the main view (Mod List vs Settings).
- Listens for Tauri events (`install_progress`, `install_log`).

**`src/components/ModDetailsPanel.jsx`**
- Displays mod metadata.
- Provides controls for **Priority Editing** (Input number -> `set_mod_priority`).
- Allows toggling flags (Fix Mesh, etc.) before installation.

## Build & Run Instructions

### Prerequisites
- **Node.js** (for frontend dependencies)
- **Rust** (cargo)
- **Visual Studio Build Tools** (C++ workloads for native compilation)

### Development
To run the app with hot-reloading:
1.  Run **`run_dev.bat`** (or `npx tauri dev` in `repak-gui`).

### Production Build
To create a standalone executable:
1.  Run **`build_project.bat`**.
2.  The output executable will be in `repak-gui/target/release/`.

## IPC API Reference (Tauri Commands)

| Command | Description |
|---------|-------------|
| `parse_dropped_files` | Analyzes incoming files for mod type and required fixes. |
| `install_mods` | Starts the async installation process. |
| `get_pak_files` | Scans `~mods` and returns a list of installed mods with metadata. |
| `set_mod_priority` | Renames a mod to update its loading priority (number of 9s). |
| `toggle_mod` | Enables/Disables a mod by renaming extension. |
| `start_file_watcher` | Activates the directory watcher. |
| `check_for_updates` | Queries GitHub for the latest release. |
