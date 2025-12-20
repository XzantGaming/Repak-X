# Bundled LOD Disabler Mod - JSX Changes

This document contains the JSX changes needed to support the bundled LOD Disabler mod feature.

## Overview

The Rust backend has been updated to:
1. Bundle the LOD Disabler .pak file into the executable (via feature flag)
2. Auto-deploy it to `~mods/_LOD-Disabler (Built-in)/` when a valid mods folder is detected
3. Expose Tauri commands: `check_lod_disabler_deployed`, `get_lod_disabler_path`, `deploy_lod_disabler`

## Changes to `ToolsPanel.jsx`

### 1. Update LOD Disabler Mod Detection

Replace the existing `lodDisablerMod` useMemo (around line 25-32):

```jsx
// Find LOD Disabler mod
const lodDisablerMod = useMemo(() => {
    return mods.find(mod => {
        const modName = mod.custom_name || mod.path?.split('\\').pop() || '';
        return modName.toLowerCase().includes('lods_disabler') ||
            mod.path?.toLowerCase().includes('lods_disabler');
    });
}, [mods]);
```

**Replace with:**

```jsx
// Find LOD Disabler mod - prioritize bundled mod in _LOD-Disabler folder
const lodDisablerMod = useMemo(() => {
    // First look for the bundled mod in the special folder
    const bundledMod = mods.find(mod => {
        const modPath = mod.path?.toLowerCase() || '';
        return modPath.includes('_lod-disabler') && modPath.includes('lods_disabler');
    });
    if (bundledMod) return bundledMod;
    
    // Fallback to any LOD disabler mod
    return mods.find(mod => {
        const modName = mod.custom_name || mod.path?.split('\\').pop() || '';
        return modName.toLowerCase().includes('lods_disabler') ||
            mod.path?.toLowerCase().includes('lods_disabler');
    });
}, [mods]);
```

### 2. Add Bundled Mod Detection

Add this new useMemo after `lodDisablerMod`:

```jsx
// Check if this is the bundled mod
const isBundledMod = useMemo(() => {
    if (!lodDisablerMod) return false;
    const modPath = lodDisablerMod.path?.toLowerCase() || '';
    return modPath.includes('_lod-disabler');
}, [lodDisablerMod]);
```

### 3. Update Display Name Logic

Replace the existing `lodModDisplayName` useMemo:

```jsx
// Get display name for LOD Disabler mod
const lodModDisplayName = useMemo(() => {
    if (!lodDisablerMod) return '';
    return lodDisablerMod.custom_name || lodDisablerMod.path?.split('\\').pop() || 'Unknown';
}, [lodDisablerMod]);
```

**Replace with:**

```jsx
// Get display name for LOD Disabler mod
const lodModDisplayName = useMemo(() => {
    if (!lodDisablerMod) return '';
    if (isBundledMod) return 'LOD Disabler (Built-in)';
    return lodDisablerMod.custom_name || lodDisablerMod.path?.split('\\').pop() || 'Unknown';
}, [lodDisablerMod, isBundledMod]);
```

### 4. Update UI Text for Bundled Mod Status

In the "Character LODs Thanos" section, update the mod info text (around line 306-308):

**Replace:**
```jsx
<p style={{ fontSize: '0.75rem', opacity: 0.5, marginTop: '0.5rem' }}>
    Mod: {lodModDisplayName}
</p>
```

**With:**
```jsx
<p style={{ fontSize: '0.75rem', opacity: 0.5, marginTop: '0.5rem' }}>
    {isBundledMod ? '✓ Built-in mod (auto-deployed)' : `Mod: ${lodModDisplayName}`}
</p>
```

### 5. Update "Not Found" Message

Update the message shown when no LOD Disabler mod is found (around line 312-316):

**Replace:**
```jsx
<p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
    No LOD Disabler mod found. Install the <span
        onClick={() => open('https://www.nexusmods.com/marvelrivals/mods/5303')}
        style={{ color: 'var(--accent-primary)', textDecoration: 'underline', cursor: 'pointer' }}
    >Character LODs Disabler</span> mod to use this feature.
</p>
```

**With:**
```jsx
<p style={{ fontSize: '0.9rem', opacity: 0.7, marginBottom: '0.5rem' }}>
    LOD Disabler not found. This mod is bundled with the app and should be auto-deployed when you set a valid mods folder.
    If missing, try re-selecting your mods folder.
</p>
```

## Complete Modified Section

Here's the complete modified section for reference:

```jsx
// Find LOD Disabler mod - prioritize bundled mod in _LOD-Disabler folder
const lodDisablerMod = useMemo(() => {
    // First look for the bundled mod in the special folder
    const bundledMod = mods.find(mod => {
        const modPath = mod.path?.toLowerCase() || '';
        return modPath.includes('_lod-disabler') && modPath.includes('lods_disabler');
    });
    if (bundledMod) return bundledMod;
    
    // Fallback to any LOD disabler mod
    return mods.find(mod => {
        const modName = mod.custom_name || mod.path?.split('\\').pop() || '';
        return modName.toLowerCase().includes('lods_disabler') ||
            mod.path?.toLowerCase().includes('lods_disabler');
    });
}, [mods]);

// Check if this is the bundled mod
const isBundledMod = useMemo(() => {
    if (!lodDisablerMod) return false;
    const modPath = lodDisablerMod.path?.toLowerCase() || '';
    return modPath.includes('_lod-disabler');
}, [lodDisablerMod]);

// Get display name for LOD Disabler mod
const lodModDisplayName = useMemo(() => {
    if (!lodDisablerMod) return '';
    if (isBundledMod) return 'LOD Disabler (Built-in)';
    return lodDisablerMod.custom_name || lodDisablerMod.path?.split('\\').pop() || 'Unknown';
}, [lodDisablerMod, isBundledMod]);
```

## New Tauri Commands Available

The following Tauri commands are now available from the Rust backend:

```javascript
// Check if the bundled LOD mod is deployed
const isDeployed = await invoke('check_lod_disabler_deployed');

// Get the path to the LOD mod
const path = await invoke('get_lod_disabler_path');

// Manually deploy the bundled LOD mod (if needed)
const wasDeployed = await invoke('deploy_lod_disabler');
```

## How It Works

1. When the user sets a valid mods folder (via auto-detect or manual selection), the Rust backend automatically deploys the bundled LOD Disabler mod to `~mods/_LOD-Disabler (Built-in)/SK_LODs_Disabler_9999999_P.pak`

2. The mod is placed in a special folder with underscore prefix (`_LOD-Disabler`) so it sorts to the top and is clearly marked as built-in

3. The JSX changes prioritize finding this bundled mod over any user-installed LOD disabler mods

4. The UI shows "✓ Built-in mod (auto-deployed)" when the bundled mod is detected

## Building with Bundled Mod

To include the LOD Disabler mod in the build:

1. Download from https://www.nexusmods.com/marvelrivals/mods/5303
2. Place at `repak-gui/src/bundled_mods/SK_LODs_Disabler_9999999_P.pak`
3. Build with feature flag:
   ```bash
   cargo build --release --features bundled_lod_mod
   ```

Without the feature flag, the app compiles normally but won't auto-deploy the LOD mod.
