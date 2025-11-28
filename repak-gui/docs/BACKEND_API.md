# Backend API Documentation

This document describes the Tauri backend commands available for the frontend to use.

## Table of Contents

- [USMAP Management](#usmap-management)
- [Character Data](#character-data)
- [Storage Locations](#storage-locations)
- [Data Structures](#data-structures)
- [Events](#events)

---

## USMAP Management

USMAP files are required for processing unversioned Unreal Engine assets. All USMAP files are stored in the user's roaming folder.

**Storage Location:** `%APPDATA%/RepakGuiRevamped/Usmap/`

> **Note:** Only one USMAP file should exist at a time. When a new file is copied, existing files are automatically deleted.

### Commands

#### `copy_usmap_to_folder`

Copy a new USMAP file to the roaming folder, replacing any existing USMAP files.

```typescript
await invoke('copy_usmap_to_folder', { sourcePath: string }): Promise<string>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `sourcePath` | `string` | Full path to the source .usmap file |

**Returns:** The filename of the copied USMAP file (just the name, not full path)

**Behavior:**
- Deletes ALL existing .usmap files in the roaming Usmap folder before copying
- Copies the new file to `%APPDATA%/RepakGuiRevamped/Usmap/`

---

#### `set_usmap_path`

Save the USMAP filename to the application state.

```typescript
await invoke('set_usmap_path', { usmapPath: string }): Promise<void>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `usmapPath` | `string` | The USMAP filename to save |

---

#### `get_usmap_path`

Get the saved USMAP filename from application state.

```typescript
await invoke('get_usmap_path'): Promise<string>
```

**Returns:** The saved USMAP filename (empty string if not set)

---

#### `get_usmap_dir_path`

Get the full path to the USMAP directory.

```typescript
await invoke('get_usmap_dir_path'): Promise<string>
```

**Returns:** Full path to `%APPDATA%/RepakGuiRevamped/Usmap/`

---

#### `list_usmap_files`

List all .usmap files currently in the roaming Usmap folder. Reads from filesystem at runtime, not from saved state.

```typescript
await invoke('list_usmap_files'): Promise<string[]>
```

**Returns:** Array of filenames (not full paths) of .usmap files in the folder

---

#### `get_current_usmap_file`

Get the currently active USMAP file by reading from filesystem.

```typescript
await invoke('get_current_usmap_file'): Promise<string>
```

**Returns:** 
- Filename of the first .usmap file found (there should only be one)
- Empty string if no .usmap files exist

---

#### `get_current_usmap_full_path`

Get the full path to the currently active USMAP file.

```typescript
await invoke('get_current_usmap_full_path'): Promise<string>
```

**Returns:**
- Full path to the .usmap file if one exists
- Empty string if no .usmap file exists

---

#### `delete_current_usmap`

Delete the currently active USMAP file from the roaming folder.

```typescript
await invoke('delete_current_usmap'): Promise<boolean>
```

**Returns:**
- `true` if a file was deleted
- `false` if no file existed to delete

---

## Character Data

Character and skin data is used for mod type detection and identification. The data is fetched from [rivalskins.com](https://rivalskins.com) and cached locally.

**Storage Location:** `%APPDATA%/RepakGuiRevamped/character_data.json`

### Commands

#### `get_character_data`

Get all cached character/skin data.

```typescript
await invoke('get_character_data'): Promise<CharacterSkin[]>
```

**Returns:** Array of all character skins in the database

---

#### `get_character_by_skin_id`

Fast lookup of character info by skin ID.

```typescript
await invoke('get_character_by_skin_id', { skinId: string }): Promise<CharacterSkin | null>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `skinId` | `string` | The 7-digit skin ID (e.g., "1011001") |

**Returns:** The matching CharacterSkin or null if not found

---

#### `update_character_data_from_rivalskins`

Fetch new skin data from rivalskins.com and merge with existing data. Runs in background and emits progress to the install log.

```typescript
await invoke('update_character_data_from_rivalskins'): Promise<number>
```

**Returns:** Number of new entries added (or throws "Cancelled" if cancelled)

**Behavior:**
1. Fetches all costume links from rivalskins.com
2. For each skin page, scrapes the actual 7-digit skin ID from the HTML
   - Handles IDs with "ps" prefix (e.g., `ps1050504` → `1050504`)
   - For default skins where ID isn't displayed: uses `{character_id}001`
   - Skips non-default skins if no ID can be found
   - **Checks for cancellation between each page fetch**
3. Merges new skins with existing data (preserves existing, adds new)
4. Sorts by character ID (numeric) then skin ID (numeric)
5. Saves to the JSON file
6. Refreshes the in-memory cache

**Events:** Emits `install_log` events with progress updates (prefixed with `[Character Data]`)

**Cancellation:** Can be cancelled via `cancel_character_update` command

---

#### `cancel_character_update`

Cancel an ongoing character data update.

```typescript
await invoke('cancel_character_update'): Promise<void>
```

**Behavior:** Sets a cancellation flag that the update process checks between page fetches. The running `update_character_data_from_rivalskins` will return an error with message "Cancelled".

---

#### `identify_mod_character`

Try to identify the character/skin from a mod's file paths.

```typescript
await invoke('identify_mod_character', { filePaths: string[] }): Promise<[string, string] | null>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `filePaths` | `string[]` | Array of file paths from the mod |

**Returns:** Tuple of `[character_name, skin_name]` or null if not identified

---

#### `get_character_data_path`

Get the full path to the character data JSON file.

```typescript
await invoke('get_character_data_path'): Promise<string>
```

**Returns:** Full path to `%APPDATA%/RepakGuiRevamped/character_data.json`

---

#### `refresh_character_cache`

Force reload the character data from disk into memory cache.

```typescript
await invoke('refresh_character_cache'): Promise<void>
```

---

## Storage Locations

All application data is stored in the user's roaming folder:

| Path | Description |
|------|-------------|
| `%APPDATA%/RepakGuiRevamped/` | Application root directory |
| `%APPDATA%/RepakGuiRevamped/state.json` | Application settings and state |
| `%APPDATA%/RepakGuiRevamped/character_data.json` | Character/skin database |
| `%APPDATA%/RepakGuiRevamped/Usmap/` | USMAP files directory |

---

## Data Structures

### CharacterSkin

```typescript
interface CharacterSkin {
  name: string;      // Character name (e.g., "Hulk", "Spider-Man")
  id: string;        // Character ID - 4 digits (e.g., "1011", "1036")
  skinid: string;    // Skin ID - 7 digits (e.g., "1011001", "1036500")
  skin_name: string; // Skin display name (e.g., "Default", "Symbiote")
}
```

### Skin ID Format

Skin IDs are 7-digit identifiers scraped from game files via rivalskins.com:
- **First 4 digits:** Character ID (e.g., `1011` = Hulk, `1036` = Spider-Man)
- **Last 3 digits:** Skin variant number (assigned by the game, not predictable)

**Default skins** always use `001` as the suffix (e.g., `1011001` for Hulk Default).

**Examples:**
- `1011001` = Hulk Default
- `1036001` = Spider-Man Default  
- `1049301` = Wolverine skin
- `1050504` = Invisible Woman skin

**Technical Notes:**
- Some IDs on the website have a `ps` prefix (e.g., `ps1050504`) which is automatically stripped
- Default skin IDs aren't always displayed on rivalskins.com, so the scraper generates them using `{character_id}001`
- Non-default skins without a visible ID are skipped (not guessed)

---

## Events

### `character_update_log`

Emitted during `update_character_data_from_rivalskins` to provide progress updates.

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('character_update_log', (event) => {
  console.log('Update progress:', event.payload); // string message
});

// Remember to cleanup
unlisten();
```

**Payload:** `string` - Progress message (e.g., "Connecting to rivalskins.com...", "Found 150 costume links to process")

---

## Example Usage

### Setting up USMAP

```typescript
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// Let user select a USMAP file
const selected = await open({
  filters: [{ name: 'USmap Files', extensions: ['usmap'] }],
  title: 'Select USmap File'
});

if (selected) {
  // Copy to roaming folder (deletes existing)
  const filename = await invoke('copy_usmap_to_folder', { sourcePath: selected });
  
  // Save to app state
  await invoke('set_usmap_path', { usmapPath: filename });
  
  console.log(`USMAP set to: ${filename}`);
}
```

### Updating Character Database

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Listen for progress updates
const unlisten = await listen('character_update_log', (event) => {
  console.log(event.payload);
});

try {
  const newCount = await invoke('update_character_data_from_rivalskins');
  console.log(`Added ${newCount} new skins`);
  
  // Get updated data
  const allSkins = await invoke('get_character_data');
  console.log(`Total skins in database: ${allSkins.length}`);
} finally {
  unlisten();
}
```

### Looking up Character Info

```typescript
import { invoke } from '@tauri-apps/api/core';

// Lookup by skin ID
const skin = await invoke('get_character_by_skin_id', { skinId: '1011001' });
if (skin) {
  console.log(`${skin.name} - ${skin.skin_name}`); // "Hulk - Default"
}

// Identify from mod files
const result = await invoke('identify_mod_character', { 
  filePaths: ['/Game/Characters/Hero/1011/Meshes/SK_Hulk.uasset'] 
});
if (result) {
  const [charName, skinName] = result;
  console.log(`Detected: ${charName} - ${skinName}`);
}
```
