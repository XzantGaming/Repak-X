# Mod Detection API Documentation

## Overview

This document describes how to use the mod detection API to display mod information in the frontend UI. The API provides detailed information about mod types and character associations.

---

## API Endpoint

### `get_mod_details`

**Tauri Command:** `get_mod_details`

**Description:** Retrieves detailed information about a mod file, including character name and mod category separated for display in different UI boxes.

**Parameters:**
- `mod_path: string` - Full path to the mod file (.pak or .utoc)

**Returns:** `ModDetails` object

---

## Response Structure

### `ModDetails` Object

```typescript
interface ModDetails {
  mod_name: string;           // Mod filename without extension
  mod_type: string;           // Combined format: "Character - Category" or just "Category"
  character_name: string;     // Character name only (empty if no character)
  category: string;           // Mod category only (e.g., "Audio", "Mesh", "VFX")
  file_count: number;         // Number of files in the mod
  total_size: number;         // Total size in bytes
  files: string[];            // Array of file paths inside the mod
  is_iostore: boolean;        // Whether this is an IoStore mod
}
```

---

## Field Descriptions

### `mod_type`
The combined string using " - " (space-hyphen-space) as separator.

**Format:**
- With character: `"Character - Category"`
- Without character: `"Category"`
- Multiple heroes: `"Multiple Heroes (N) - Category"`

**Examples:**
- `"Blade - Audio"`
- `"Hawkeye - Default - Mesh"`
- `"Psylocke - VFX"`
- `"Multiple Heroes (2) - Audio"`
- `"Audio"` (no character detected)

### `character_name`
The character name extracted from the mod, or empty string if no character detected.

**Examples:**
- `"Blade"`
- `"Hawkeye - Default"` (includes skin name)
- `"Psylocke"`
- `""` (empty for mods without character)

### `category`
The pure mod category without character information.

**Possible Values:**
- `"Audio"` - Audio/voice mods
- `"Mesh"` - Skeletal mesh mods (character models)
- `"Static Mesh"` - Static mesh mods (props, environment)
- `"VFX"` - Visual effects mods
- `"UI"` - User interface mods
- `"Movies"` - Video/cutscene mods
- `"Retexture"` - Texture mods
- `"Unknown"` - Could not determine type

---

## Usage Examples

### Example 1: Calling the API

```javascript
import { invoke } from '@tauri-apps/api/core';

async function getModDetails(modPath) {
  try {
    const details = await invoke('get_mod_details', { modPath });
    return details;
  } catch (error) {
    console.error('Failed to get mod details:', error);
    return null;
  }
}
```

### Example 2: Displaying in Separate Boxes

```jsx
function ModDetailsDisplay({ modPath }) {
  const [details, setDetails] = useState(null);

  useEffect(() => {
    invoke('get_mod_details', { modPath })
      .then(setDetails)
      .catch(console.error);
  }, [modPath]);

  if (!details) return <div>Loading...</div>;

  return (
    <div className="mod-details">
      <div className="mod-name-box">
        <h3>{details.mod_name}</h3>
      </div>
      
      {/* Character Box - Only show if character exists */}
      {details.character_name && (
        <div className="character-box">
          <label>Character</label>
          <span>{details.character_name}</span>
        </div>
      )}
      
      {/* Category Box */}
      <div className="category-box">
        <label>Mod Type</label>
        <span>{details.category}</span>
      </div>
      
      {/* Or use combined mod_type */}
      <div className="combined-box">
        <label>Full Type</label>
        <span>{details.mod_type}</span>
      </div>
      
      <div className="info-box">
        <p>Files: {details.file_count}</p>
        <p>Size: {formatBytes(details.total_size)}</p>
      </div>
    </div>
  );
}
```

### Example 3: Splitting mod_type Manually (Alternative)

If you prefer to split the `mod_type` string yourself:

```javascript
function parseModType(modType) {
  const parts = modType.split(' - ');
  
  if (parts.length > 1) {
    // Has character
    return {
      character: parts[0],
      category: parts[parts.length - 1]
    };
  } else {
    // No character
    return {
      character: '',
      category: parts[0]
    };
  }
}

// Usage
const { character, category } = parseModType(details.mod_type);
```

### Example 4: Conditional Rendering Based on Character

```jsx
function ModTypeDisplay({ details }) {
  return (
    <div className="mod-type-display">
      {details.character_name ? (
        // Character-specific mod
        <>
          <div className="character-badge">{details.character_name}</div>
          <div className="category-badge">{details.category}</div>
        </>
      ) : (
        // Generic mod
        <div className="category-badge-large">{details.category}</div>
      )}
    </div>
  );
}
```

---

## Real-World Examples

### Example Response 1: Character Audio Mod

```json
{
  "mod_name": "VergilSFX_9999999",
  "mod_type": "Blade - Audio",
  "character_name": "Blade",
  "category": "Audio",
  "file_count": 5,
  "total_size": 11234567,
  "files": [
    "Marvel/Content/NwiseAudio/English(US)/bnk_vo_1044001.bnk",
    "Marvel/Content/NwiseAudio/English(US)/bnk_vo_system.bnk",
    "Marvel/Content/NwiseAudio/bnk_sfx_1044001.bnk"
  ],
  "is_iostore": false
}
```

**Display:**
- Character Box: `"Blade"`
- Category Box: `"Audio"`

---

### Example Response 2: Skin-Specific Mesh Mod

```json
{
  "mod_name": "Hawkeye_Default_Mesh",
  "mod_type": "Hawkeye - Default - Mesh",
  "character_name": "Hawkeye - Default",
  "category": "Mesh",
  "file_count": 12,
  "total_size": 45678901,
  "files": [
    "Marvel/Content/Marvel/Characters/1021/1021001/SK_Hawkeye.uasset",
    "Marvel/Content/Marvel/Characters/1021/1021001/SK_Hawkeye.uexp"
  ],
  "is_iostore": true
}
```

**Display:**
- Character Box: `"Hawkeye - Default"`
- Category Box: `"Mesh"`

---

### Example Response 3: Multiple Heroes VFX Mod

```json
{
  "mod_name": "TeamVFX_Mod",
  "mod_type": "Multiple Heroes (3) - VFX",
  "character_name": "",
  "category": "VFX",
  "file_count": 25,
  "total_size": 78901234,
  "files": [
    "Marvel/Content/Marvel/VFX/Characters/1021/MI_Effect.uasset",
    "Marvel/Content/Marvel/VFX/Characters/1048/MI_Effect.uasset",
    "Marvel/Content/Marvel/VFX/Characters/1044/MI_Effect.uasset"
  ],
  "is_iostore": false
}
```

**Display:**
- Character Box: Not shown (empty character_name)
- Category Box: `"VFX"`
- Or show: `"Multiple Heroes (3) - VFX"` in a single box

---

### Example Response 4: Generic UI Mod

```json
{
  "mod_name": "CustomUI_Mod",
  "mod_type": "UI",
  "character_name": "",
  "category": "UI",
  "file_count": 8,
  "total_size": 5678901,
  "files": [
    "Marvel/Content/Marvel/UI/HUD/T_CustomIcon.uasset",
    "Marvel/Content/Marvel/UI/Menu/T_Background.uasset"
  ],
  "is_iostore": false
}
```

**Display:**
- Character Box: Not shown (empty character_name)
- Category Box: `"UI"`

---

## UI Design Recommendations

### Layout Option 1: Separate Boxes (Recommended)

```
┌─────────────────────────────────────┐
│  Mod Name: VergilSFX_9999999        │
├─────────────────┬───────────────────┤
│  CHARACTER      │  MOD TYPE         │
│  Blade          │  Audio            │
└─────────────────┴───────────────────┘
```

### Layout Option 2: Inline Display

```
┌─────────────────────────────────────┐
│  VergilSFX_9999999                  │
│  [Blade] Audio                      │
└─────────────────────────────────────┘
```

### Layout Option 3: Badge Style

```
┌─────────────────────────────────────┐
│  VergilSFX_9999999                  │
│  🎭 Blade  │  🔊 Audio              │
└─────────────────────────────────────┘
```

---

## Category Icons (Optional)

You can use icons to represent different mod categories:

- **Audio**: 🔊 or 🎵
- **Mesh**: 🎭 or 👤
- **VFX**: ✨ or 🎆
- **UI**: 🖥️ or 📱
- **Movies**: 🎬 or 📹
- **Retexture**: 🎨 or 🖼️
- **Static Mesh**: 🏗️ or 📦

---

## Character Detection

### How It Works

The system detects characters from:

1. **File paths** containing character IDs (e.g., `/Characters/1044/`)
2. **Filenames** with character IDs (e.g., `bnk_vo_1044001.bnk`)
3. **Skin IDs** for specific character skins (e.g., `1021/1021001`)

### Supported Characters

The system supports all Marvel Rivals characters including:
- Blade (1044)
- Psylocke (1048)
- Hawkeye (1021)
- Iron Man (1034)
- And many more...

---

## Error Handling

```javascript
async function safeGetModDetails(modPath) {
  try {
    const details = await invoke('get_mod_details', { modPath });
    return details;
  } catch (error) {
    console.error('Error getting mod details:', error);
    
    // Return fallback data
    return {
      mod_name: 'Unknown Mod',
      mod_type: 'Unknown',
      character_name: '',
      category: 'Unknown',
      file_count: 0,
      total_size: 0,
      files: [],
      is_iostore: false
    };
  }
}
```

---

## Notes

- The `character_name` field will be an empty string (`""`) if no character is detected
- The `mod_type` field always contains a value (never empty)
- The " - " separator is consistent and can be reliably used for splitting
- Multiple heroes mods will have empty `character_name` but `mod_type` will show "Multiple Heroes (N) - Category"
- IoStore mods (`.utoc`/`.ucas`) are automatically detected and handled

---

## Questions?

If you need additional fields or have questions about the API, please contact the backend team.
