# Frontend Changes Needed for New Backend Features

This document outlines the JSX/frontend changes needed to integrate the new backend features.

---

## 1. Update Mod Feature

### Backend Command
```javascript
await invoke('update_mod', {
  oldModPath: string,      // Path to the existing mod to replace
  newModSource: string,    // Path to new mod files (.pak, .zip, .rar, .7z, or directory)
  preserveName: boolean    // If true, keeps old mod's name; if false, uses new mod's name
});
```

### Returns
```typescript
interface UpdateModResult {
  new_mod_path: string;
  old_mod_deleted: boolean;
  preserved_enabled_state: boolean;
  preserved_folder: string | null;
}
```

### Suggested Frontend Integration Points

1. **Context Menu** (`ContextMenu.jsx`):
   - Add "Update/Replace Mod..." option that opens a file picker
   - On selection, call `invoke('update_mod', { oldModPath: mod.path, newModSource: selectedPath, preserveName: true })`

2. **Drag & Drop** (`DropZoneOverlay.jsx` or `App.jsx`):
   - When dropping a file onto an existing mod card, show confirmation dialog
   - Ask user if they want to replace the mod
   - Call `update_mod` with the dropped file as `newModSource`

3. **Mod Details Panel** (`ModDetailsPanel.jsx`):
   - Add "Update Mod" button that opens file picker

---

## 2. Auto App Update Feature

### Backend Commands

```javascript
// Check for updates (already exists)
const updateInfo = await invoke('check_for_updates');
// Returns: { latest: string, url: string, asset_url?: string, asset_name?: string } | null

// Download update with progress
const downloadedPath = await invoke('download_update', {
  assetUrl: updateInfo.asset_url,
  assetName: updateInfo.asset_name
});

// Apply update (creates updater script, runs on app close)
await invoke('apply_update', { downloadedPath });

// Cancel download and cleanup
await invoke('cancel_update_download');

// Get/set auto-update preference
const enabled = await invoke('get_auto_update_enabled');
await invoke('set_auto_update_enabled', { enabled: true });
```

### Events to Listen For
```javascript
// Download progress
listen('update_download_progress', (event) => {
  const { downloaded_bytes, total_bytes, percentage, status } = event.payload;
  // status: "downloading" | "extracting" | "ready" | "error"
});

// Update ready to apply (app will update on close)
listen('update_ready_to_apply', () => {
  // Show notification that update will apply on restart
});
```

### Suggested Frontend Integration Points

1. **Settings Panel** (`SettingsPanel.jsx`):
   - Add checkbox for "Auto-check for updates on startup"
   - Add "Check for Updates" button

2. **App.jsx or TitleBar.jsx**:
   - On startup, if `auto_check_updates` is enabled, call `check_for_updates()`
   - If update available, show notification/modal with:
     - Version info
     - "Download & Install" button
     - "Remind me later" button
     - Download progress bar

3. **Update Modal Component** (new):
   - Show download progress
   - "Cancel" button during download
   - After download: "Restart to Apply" button (closes app, updater runs)

---

## 3. Discord Rich Presence

### Backend Commands

```javascript
// Connect/disconnect
await invoke('discord_connect');
await invoke('discord_disconnect');

// Check status
const isConnected = await invoke('discord_is_connected');

// Set activity states
await invoke('discord_set_idle');
await invoke('discord_set_managing_mods', { modCount: 42 });
await invoke('discord_set_installing', { modName: "Cool Skin Mod" });
await invoke('discord_set_sharing');
await invoke('discord_set_receiving');
await invoke('discord_clear_activity');
```

### Suggested Frontend Integration Points

1. **Settings Panel** (`SettingsPanel.jsx`):
   - Add toggle for "Enable Discord Rich Presence"
   - On toggle: call `discord_connect()` or `discord_disconnect()`

2. **App.jsx** (automatic activity updates):
   - On mod list load: `discord_set_managing_mods({ modCount: mods.length })`
   - During installation: `discord_set_installing({ modName })`
   - When P2P sharing: `discord_set_sharing()`
   - When P2P receiving: `discord_set_receiving()`

3. **Startup** (`App.jsx`):
   - Check saved preference, if enabled call `discord_connect()`
   - Set initial activity with mod count

### Theme-Based Logo Switching

The Discord logo can change based on the app's color palette. When the user changes their color theme in settings, call:

```javascript
await invoke('discord_set_theme', { theme: 'blue' }); // or 'red', 'green', 'purple', etc.
```

**Supported themes** (matching app color presets):
- `red` (crimson)
- `blue` (default)
- `cyan` / `teal` (turquoise)
- `green`
- `orange`
- `pink` (magenta)

To get the current theme:
```javascript
const theme = await invoke('discord_get_theme');
```

### Note on Discord Application ID
The Discord Rich Presence requires a Discord Application ID. You need to:
1. Go to https://discord.com/developers/applications
2. Create a new application named "Repak X" (or similar)
3. Copy the Application ID
4. Update `DISCORD_APP_ID` in `discord_presence.rs`
5. Upload logo assets for each color theme in the Rich Presence section:
   - `repakx_logo` (default fallback)
   - `repakx_logo_blue`
   - `repakx_logo_red`
   - `repakx_logo_cyan`
   - `repakx_logo_green`
   - `repakx_logo_orange`
   - `repakx_logo_pink`

---

## Summary of New Tauri Commands

| Feature | Command | Description |
|---------|---------|-------------|
| Update Mod | `update_mod` | Replace existing mod with new files |
| Auto Update | `download_update` | Download update ZIP/EXE with progress |
| Auto Update | `apply_update` | Schedule update to apply on app close |
| Auto Update | `cancel_update_download` | Cancel and cleanup |
| Auto Update | `get_auto_update_enabled` | Get preference |
| Auto Update | `set_auto_update_enabled` | Set preference |
| Discord | `discord_connect` | Enable Rich Presence |
| Discord | `discord_disconnect` | Disable Rich Presence |
| Discord | `discord_is_connected` | Check connection status |
| Discord | `discord_set_idle` | Set idle activity |
| Discord | `discord_set_managing_mods` | Set mod count activity |
| Discord | `discord_set_installing` | Set installing activity |
| Discord | `discord_set_sharing` | Set P2P sharing activity |
| Discord | `discord_set_receiving` | Set P2P receiving activity |
| Discord | `discord_clear_activity` | Clear activity |
| Discord | `discord_set_theme` | Set logo color theme |
| Discord | `discord_get_theme` | Get current theme |

---

## Events

| Event | Payload | Description |
|-------|---------|-------------|
| `update_download_progress` | `{ downloaded_bytes, total_bytes, percentage, status }` | Download progress |
| `update_ready_to_apply` | `()` | Update staged, will apply on close |
