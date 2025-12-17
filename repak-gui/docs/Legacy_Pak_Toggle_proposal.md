# Skip IoStore Conversion for Non-UAsset Mods

## Overview

This proposal outlines the implementation of a feature to **skip IoStore conversion** for mods that don't contain `.uasset` files (e.g., `.bnk`, `.ini`, `.cfg` files). These mods should be installed in **legacy PAK format** instead.

---

## Problem Statement

Currently, all mods processed through the Mod Install Panel are converted to IoStore format (`.pak` + `.utoc` + `.ucas`). However, IoStore conversion is only beneficial for Unreal Engine asset files (`.uasset`, `.uexp`, `.ubulk`). 

Mods containing only non-UE files like:
- **Audio banks**: `.bnk`, `.wem`
- **Config files**: `.ini`, `.cfg`, `.json`
- **Other data**: `.txt`, `.xml`

...do not benefit from IoStore and may even have compatibility issues. These should be installed as **legacy PAK files** only.

---

## Proposed Solution

### 1. New Field in `InstallableMod` Struct

Add a new boolean field to track whether to force legacy PAK format:

```rust
// In install_mod.rs - InstallableMod struct
pub struct InstallableMod {
    // ... existing fields ...
    pub force_legacy_pak: bool,  // NEW: Skip IoStore, use legacy PAK format
}
```

**Default Implementation:**
```rust
impl Default for InstallableMod {
    fn default() -> Self {
        InstallableMod {
            // ... existing defaults ...
            force_legacy_pak: false,
        }
    }
}
```

---

### 2. Auto-Detection Function

Create a function to detect if a mod contains UAsset files:

```rust
// In install_mod.rs or utils.rs

/// Returns true if the file list contains any UAsset-related files
/// that would benefit from IoStore conversion
pub fn contains_uasset_files(files: &[String]) -> bool {
    files.iter().any(|f| {
        let lower = f.to_lowercase();
        lower.ends_with(".uasset") 
            || lower.ends_with(".uexp") 
            || lower.ends_with(".ubulk")
            || lower.ends_with(".umap")
    })
}

/// Returns true if the mod should use legacy PAK format
/// (no UAsset files detected - only .bnk, .ini, etc.)
pub fn should_use_legacy_format(files: &[String]) -> bool {
    !contains_uasset_files(files)
}
```

---

### 3. Integration Points in `map_to_mods_internal()`

Update the mod detection logic in `install_mod.rs`:

```rust
// In map_to_mods_internal() function, after file list is collected:

// Auto-detect if mod should use legacy format (no uasset files)
let auto_force_legacy = should_use_legacy_format(&files);

// ... later in InstallableMod creation:
Ok(InstallableMod {
    // ... existing fields ...
    force_legacy_pak: auto_force_legacy,
    ..Default::default()
})
```

---

### 4. Installation Logic Changes in `install_mod_logic.rs`

Modify `install_mods_in_viewport()` to respect the `force_legacy_pak` flag:

```rust
// In install_mods_in_viewport() - around line 186 where is_dir handling occurs

if installable_mod.is_dir {
    // Copy source directory to temp dir to avoid modifying original files
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to create temp directory: {}", e);
            continue;
        }
    };
    let temp_path = temp_dir.path().to_path_buf();
    
    // Copy all files from source to temp
    let source_path = PathBuf::from(&installable_mod.mod_path);
    if let Err(e) = copy_dir_recursive(&source_path, &temp_path) {
        error!("Failed to copy mod files to temp directory: {}", e);
        continue;
    }
    info!("Copied mod files to temp directory for processing");
    
    // NEW: Check if we should skip IoStore conversion
    if installable_mod.force_legacy_pak {
        info!("Force legacy PAK enabled - skipping IoStore conversion for: {}", installable_mod.mod_name);
        let res = repak_dir(
            installable_mod,
            temp_path,
            mod_directory.to_path_buf(),
            installed_mods_ptr,
        );
        if let Err(e) = res {
            error!("Failed to create legacy PAK: {}", e);
        } else {
            info!("Installed mod as legacy PAK: {}", installable_mod.mod_name);
        }
    } else {
        // Existing IoStore conversion path
        let res = convert_to_iostore_directory(
            installable_mod,
            mod_directory.to_path_buf(),
            temp_path,
            installed_mods_ptr,
        );
        if let Err(e) = res {
            error!("Failed to create repak from pak: {}", e);
        } else {
            info!("Installed mod: {}", installable_mod.mod_name);
        }
    }
}
```

---

### 5. Similar Logic for PAK File Repacking

In `create_repak_from_pak()` or the repak flow, add similar logic:

```rust
// When processing PAK files with repak enabled
if installable_mod.force_legacy_pak {
    // Use repak_dir() which creates legacy PAK format
    // Skip action_to_zen() IoStore conversion
}
```

---

### 6. Frontend Integration (JSX Reference)

The frontend should add a new toggle **next to the "Send to Repak" toggle** in the footer section of each mod card.

**New Setting in `buildInitialSettings`:**
```javascript
// In buildInitialSettings function
acc[idx] = {
  // ... existing settings ...
  forceLegacy: mod.auto_force_legacy || false,  // NEW
}
```

**New Toggle Definition:**
```javascript
// This toggle should appear next to the "Send to Repak" toggle
// in the install-mod-card__footer section
{
  key: 'forceLegacy',
  label: 'Legacy PAK Format',
  hint: 'Use when making Audio/Config mods (mods that dont contain uassets)'
}
```

**Toggle Locking Behavior:**

When `forceLegacy` is enabled, **all patch toggles should be disabled/locked** since patches only apply to uasset-based mods:

```javascript
// In the toggleDefinitions rendering, check if forceLegacy is enabled
const isLegacyMode = modSettings[idx]?.forceLegacy || false

// Each patch toggle should be disabled when legacy mode is on
<Switch
  key={key}
  size="sm"
  color="primary"
  checked={isLegacyMode ? false : (modSettings[idx]?.[key] || false)}
  onChange={(value) => updateModSetting(idx, key, value)}
  isDisabled={isLegacyMode}  // LOCK when legacy mode enabled
  className={`install-toggle ${isLegacyMode ? 'locked' : ''}`}
>
  <div className="install-toggle__text">
    <span className="install-toggle__label">{label}</span>
    <span className="install-toggle__hint">
      {isLegacyMode ? 'Disabled in Legacy PAK mode' : hint}
    </span>
  </div>
</Switch>
```

**Update `updateModSetting` to clear patch settings when enabling legacy mode:**
```javascript
const updateModSetting = (idx, key, value) => {
  if (key === 'toRepak' && isRepakLocked(mods[idx])) {
    return
  }
  
  // When enabling forceLegacy, clear all patch toggles
  if (key === 'forceLegacy' && value === true) {
    setModSettings(prev => ({
      ...prev,
      [idx]: { 
        ...prev[idx], 
        [key]: value,
        fixMesh: false,
        fixTexture: false,
        fixSerializeSize: false
      }
    }))
    return
  }
  
  // Prevent enabling patch toggles when in legacy mode
  if (['fixMesh', 'fixTexture', 'fixSerializeSize'].includes(key) && modSettings[idx]?.forceLegacy) {
    return
  }
  
  setModSettings(prev => ({
    ...prev,
    [idx]: { ...prev[idx], [key]: value }
  }))
}
```

**Passing to Backend:**
```javascript
// In handleInstall function
const modsToInstall = mods.map((mod, idx) => ({
  ...mod,
  ...modSettings[idx],
  toRepak: isRepakLocked(mod) ? false : (modSettings[idx]?.toRepak || false),
  forceLegacy: modSettings[idx]?.forceLegacy || false,  // NEW
}))
```

---

## File Changes Summary

| File | Changes |
|------|---------|
| `repak-gui/src/install_mod.rs` | Add `force_legacy_pak` field to `InstallableMod` struct and default impl |
| `repak-gui/src/install_mod.rs` | Add `contains_uasset_files()` and `should_use_legacy_format()` functions |
| `repak-gui/src/install_mod.rs` | Update `map_to_mods_internal()` to auto-detect legacy format need |
| `repak-gui/src/install_mod/install_mod_logic.rs` | Add conditional logic to skip IoStore when `force_legacy_pak` is true |
| `repak-gui/src/main_tauri.rs` | Update `InstallableModInfo` struct if needed for frontend communication |
| `repak-gui/src/components/InstallModPanel.jsx` | Add "Legacy PAK Format" toggle next to "Send to Repak" |

---

## Behavior Matrix

| Mod Contents | Auto-Detect | Default Toggle State | Result |
|--------------|-------------|---------------------|--------|
| `.uasset`, `.uexp` files | Has UAssets | `forceLegacy: false` | IoStore conversion |
| Only `.bnk`, `.wem` files | No UAssets | `forceLegacy: true` | Legacy PAK only |
| Only `.ini`, `.cfg` files | No UAssets | `forceLegacy: true` | Legacy PAK only |
| Mixed (`.uasset` + `.bnk`) | Has UAssets | `forceLegacy: false` | IoStore conversion |

---

## User Override

The toggle allows users to:
1. **Force legacy PAK** for mods that auto-detected as needing IoStore (override)
2. **Force IoStore** for mods that auto-detected as legacy-only (override)

This gives full control while providing sensible defaults.

---

## Notes

- The existing `mod_type == "Audio" || mod_type == "Movies"` check in `iotoc.rs` already skips IoStore for those types - this new feature extends that logic to be more granular and user-controllable
- The toggle should be visually distinct from the "Send to Repak" toggle to avoid confusion
- Consider adding a tooltip explaining when to use this option

---

## ReCompress Button - Frontend Integration

The ReCompress button in the Tools panel needs to be wired up to the `recompress_mods` Tauri command. This command scans all installed mods and recompresses any that aren't using Oodle compression (both regular PAK files and IoStore bundles).

### Backend Command (Already Implemented)

The `recompress_mods` command is registered in `main_tauri.rs` and:
- Scans all `.pak` files in the mods directory
- For **IoStore mods** (`.utoc`/`.ucas`): Uses `retoc::is_iostore_compressed()` to check and `retoc::recompress_iostore()` to recompress
- For **regular PAK files**: Checks compression type and recompresses to Oodle if needed
- Emits `recompress_progress` events for real-time progress updates
- Returns a `RecompressResult` object with detailed statistics

### Frontend Changes Required (ToolsPanel.jsx)

**1. Add State Variables** (around line 13, after existing state declarations):

```javascript
const [isRecompressing, setIsRecompressing] = useState(false);
const [recompressStatus, setRecompressStatus] = useState('');
const [recompressResult, setRecompressResult] = useState(null);
```

**2. Add useEffect for Status Clearing** (after existing useEffects, around line 46):

```javascript
// Clear recompress status after 5 seconds
useEffect(() => {
    if (recompressStatus && !isRecompressing) {
        const timer = setTimeout(() => {
            setRecompressStatus('');
        }, 5000);
        return () => clearTimeout(timer);
    }
}, [recompressStatus, isRecompressing]);
```

**3. Replace the `handleReCompress` Function** (line 80-83):

```javascript
const handleReCompress = async () => {
    setIsRecompressing(true);
    setRecompressStatus('Scanning mods...');
    setRecompressResult(null);
    try {
        const result = await invoke('recompress_mods');
        setRecompressResult(result);
        if (result.recompressed > 0) {
            setRecompressStatus(`✓ Recompressed ${result.recompressed} mod(s)! (${result.already_oodle} already compressed)`);
        } else if (result.already_oodle === result.total_scanned) {
            setRecompressStatus('✓ All mods already use Oodle compression');
        } else if (result.total_scanned === 0) {
            setRecompressStatus('No mods found to scan');
        } else {
            setRecompressStatus(`Scanned ${result.total_scanned} mods - ${result.already_oodle} already compressed`);
        }
    } catch (error) {
        setRecompressStatus(`Error: ${error}`);
    } finally {
        setIsRecompressing(false);
    }
};
```

**4. Update the Button** (around line 173-179):

```javascript
<button
    onClick={handleReCompress}
    disabled={isRecompressing}
    style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
>
    <FaFileZipper size={16} className={isRecompressing ? 'spin-animation' : ''} />
    {isRecompressing ? 'Scanning...' : 'ReCompress'}
</button>
```

**5. Add Status Display** (after the button's parent div, around line 180):

```javascript
{recompressStatus && (
    <p style={{
        fontSize: '0.85rem',
        marginTop: '0.5rem',
        color: recompressStatus.includes('Error') ? '#ff5252' : '#4CAF50'
    }}>
        {recompressStatus}
    </p>
)}
```

**6. Optional: Add Progress Listener** (in the main useEffect or a new one):

```javascript
// Listen for recompress progress events
useEffect(() => {
    const unlisten = listen('recompress_progress', (event) => {
        const { current, total, status } = event.payload;
        setRecompressStatus(`${status} (${current}/${total})`);
    });
    
    return () => {
        unlisten.then(f => f());
    };
}, []);
```

### RecompressResult Object Structure

The backend returns:

```typescript
interface RecompressResult {
    total_scanned: number;      // Total PAK files found
    already_oodle: number;      // Already using Oodle compression
    recompressed: number;       // Successfully recompressed
    failed: number;             // Failed to recompress
    skipped_iostore: number;    // (Legacy - no longer used, IoStore is now processed)
    details: RecompressDetail[];
}

interface RecompressDetail {
    mod_name: string;
    status: 'already_oodle' | 'recompressed' | 'failed';
    original_size: number;
    new_size: number | null;
    error: string | null;
}
```

### Progress Event Payload

```typescript
interface RecompressProgress {
    current: number;  // Current mod index (1-based)
    total: number;    // Total mods to process
    status: string;   // Current operation description
}
```

---

## Contains UAssets Detection - Frontend Integration

### Backend Changes (Already Implemented)

The backend now returns a `contains_uassets` field in the `InstallableModInfo` struct returned by `parse_dropped_files`. This field indicates whether the mod contains any `.uasset`, `.uexp`, `.ubulk`, or `.umap` files.

**InstallableModInfo struct now includes:**
```typescript
interface InstallableModInfo {
    mod_name: string;
    mod_type: string;
    is_dir: boolean;
    path: string;
    auto_fix_mesh: boolean;
    auto_fix_texture: boolean;
    auto_fix_serialize_size: boolean;
    auto_to_repak: boolean;
    contains_uassets: boolean;  // NEW - true if mod has .uasset/.uexp/.ubulk/.umap files
}
```

### Frontend Changes Required (InstallModPanel.jsx)

**1. Use `contains_uassets` for Toggle Locking**

When `contains_uassets` is `false`, the following toggles should be **disabled/locked** since they only apply to UAsset-based mods:
- Fix Mesh (fixMesh)
- Fix Texture (fixTexture)  
- Fix SerializeSize (fixSerializeSize)

**2. Update Toggle Rendering**

```javascript
// In the toggle rendering section, check contains_uassets
const canApplyPatches = mod.contains_uassets !== false; // Default to true if undefined

// Each patch toggle should be disabled when no uassets
<Switch
  key={key}
  size="sm"
  color="primary"
  checked={canApplyPatches ? (modSettings[idx]?.[key] || false) : false}
  onChange={(value) => updateModSetting(idx, key, value)}
  isDisabled={!canApplyPatches}
  className={`install-toggle ${!canApplyPatches ? 'locked' : ''}`}
>
  <div className="install-toggle__text">
    <span className="install-toggle__label">{label}</span>
    <span className="install-toggle__hint">
      {!canApplyPatches ? 'No UAsset files detected' : hint}
    </span>
  </div>
</Switch>
```

**3. Update `buildInitialSettings` Function**

```javascript
// In buildInitialSettings, respect contains_uassets
acc[idx] = {
  fixMesh: mod.contains_uassets !== false ? (mod.auto_fix_mesh || false) : false,
  fixTexture: mod.contains_uassets !== false ? (mod.auto_fix_texture || false) : false,
  fixSerializeSize: mod.contains_uassets !== false ? (mod.auto_fix_serialize_size || false) : false,
  toRepak: mod.auto_to_repak || false,
  // ... other settings
}
```

**4. Update `updateModSetting` to Prevent Enabling Patches on Non-UAsset Mods**

```javascript
const updateModSetting = (idx, key, value) => {
  // Prevent enabling patch toggles when no uassets
  if (['fixMesh', 'fixTexture', 'fixSerializeSize'].includes(key) && !mods[idx]?.contains_uassets) {
    return;
  }
  
  setModSettings(prev => ({
    ...prev,
    [idx]: { ...prev[idx], [key]: value }
  }));
}
```

**5. Visual Indicator for Non-UAsset Mods**

Consider adding a visual indicator (badge or icon) to show when a mod doesn't contain UAsset files:

```javascript
{!mod.contains_uassets && (
  <span className="no-uassets-badge" title="This mod contains no UAsset files - patch options disabled">
    No UAssets
  </span>
)}
```

### Behavior Matrix

| Mod Contents | `contains_uassets` | Patch Toggles | Notes |
|--------------|-------------------|---------------|-------|
| `.uasset`, `.uexp` files | `true` | Enabled | Normal UE asset mod |
| Only `.bnk`, `.wem` files | `false` | Disabled | Audio mod |
| Only `.ini`, `.cfg` files | `false` | Disabled | Config mod |
| Mixed (`.uasset` + `.bnk`) | `true` | Enabled | Has UE assets |
| Unknown/fallback | `true` | Enabled | Safe default |

### CSS Styling (Optional)

```css
.install-toggle.locked {
  opacity: 0.5;
  cursor: not-allowed;
}

.no-uassets-badge {
  background: #666;
  color: #fff;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
  margin-left: 8px;
}
```
