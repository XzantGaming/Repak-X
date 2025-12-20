# Extract Assets Feature - JSX Changes

This document contains the JSX changes needed to enable the "Extract Assets" context menu option.

## Overview

The Rust backend has been updated to:
1. Add `extract_iostore()` public function to retoc for IoStore extraction
2. Add `extract_mod_assets` Tauri command that handles both PAK and IoStore files
3. Automatically detect file type and use appropriate extraction method

## New Tauri Command

```javascript
// Extract assets from a mod file to a destination directory
// Supports: .pak, .utoc, .ucas files
// Returns: number of files extracted
const fileCount = await invoke('extract_mod_assets', {
    modPath: '/path/to/mod.pak',  // or .utoc, .ucas
    destPath: '/path/to/destination/folder'
});
```

## Changes to `ContextMenu.jsx`

### Current Code (Disabled)

The "Extract Assets" option is currently disabled at line 198-200:

```jsx
<div className="context-menu-item disabled">
    Extract Assets
</div>
```

### Replace With

```jsx
<div className="context-menu-item" onClick={async () => {
    try {
        // Open folder picker dialog
        const { open } = await import('@tauri-apps/plugin-dialog');
        const destFolder = await open({
            directory: true,
            multiple: false,
            title: 'Select destination folder for extracted assets'
        });
        
        if (destFolder) {
            // Get the mod path - handle both PAK and IoStore
            let modPath = mod.path;
            
            // If this is an IoStore mod (folder with .utoc), find the .utoc file
            if (mod.is_iostore && mod.utoc_path) {
                modPath = mod.utoc_path;
            }
            
            const fileCount = await invoke('extract_mod_assets', {
                modPath: modPath,
                destPath: destFolder
            });
            
            console.log(`Extracted ${fileCount} files to ${destFolder}`);
            
            // Optionally show success notification
            // You can add a toast/notification here
        }
    } catch (e) {
        console.error('Failed to extract assets:', e);
        // Optionally show error notification
    }
    onClose();
}}>
    Extract Assets
</div>
```

## Complete Modified Section

Here's the complete context menu section with the Extract Assets option enabled:

```jsx
<div className="context-menu-separator" />

<div className="context-menu-item" onClick={async () => {
    try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const destFolder = await open({
            directory: true,
            multiple: false,
            title: 'Select destination folder for extracted assets'
        });
        
        if (destFolder) {
            let modPath = mod.path;
            if (mod.is_iostore && mod.utoc_path) {
                modPath = mod.utoc_path;
            }
            
            const fileCount = await invoke('extract_mod_assets', {
                modPath: modPath,
                destPath: destFolder
            });
            
            console.log(`Extracted ${fileCount} files to ${destFolder}`);
        }
    } catch (e) {
        console.error('Failed to extract assets:', e);
    }
    onClose();
}}>
    Extract Assets
</div>

<div className="context-menu-item" onClick={async () => {
    try {
        await invoke('open_in_explorer', { path: mod.path });
    } catch (e) {
        console.error('Failed to open in explorer:', e);
    }
    onClose();
}}>
    Open in Explorer
</div>
```

## Optional Enhancements

### 1. Add Loading State

You may want to add a loading indicator while extraction is in progress:

```jsx
const [isExtracting, setIsExtracting] = useState(false);

// In the onClick handler:
setIsExtracting(true);
try {
    const fileCount = await invoke('extract_mod_assets', { ... });
    // Show success
} catch (e) {
    // Show error
} finally {
    setIsExtracting(false);
}
```

### 2. Add Success/Error Toast

```jsx
// After successful extraction:
window.dispatchEvent(new CustomEvent('show-toast', {
    detail: {
        message: `Successfully extracted ${fileCount} files`,
        type: 'success'
    }
}));

// On error:
window.dispatchEvent(new CustomEvent('show-toast', {
    detail: {
        message: `Failed to extract: ${e}`,
        type: 'error'
    }
}));
```

### 3. Open Extracted Folder After Completion

```jsx
const fileCount = await invoke('extract_mod_assets', {
    modPath: modPath,
    destPath: destFolder
});

// Open the extracted folder in explorer
const modName = modPath.split(/[/\\]/).pop().replace(/\.[^.]+$/, '');
await invoke('open_in_explorer', { path: `${destFolder}/${modName}` });
```

## Supported File Types

The `extract_mod_assets` command automatically handles:

| Extension | Type | Extraction Method |
|-----------|------|-------------------|
| `.pak` | PAK file | repak extraction |
| `.utoc` | IoStore TOC | retoc extraction |
| `.ucas` | IoStore CAS | Finds corresponding .utoc and extracts |

## Notes

1. The extraction creates a subfolder named after the mod file (e.g., `MyMod_P.pak` → `MyMod_P/`)
2. IoStore extraction uses the directory index to map chunks to file paths
3. Both PAK and IoStore extraction use the Marvel Rivals AES key automatically
4. The command returns the number of files extracted, which can be used for user feedback
