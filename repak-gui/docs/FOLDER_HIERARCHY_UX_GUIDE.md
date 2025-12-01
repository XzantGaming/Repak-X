# Folder Hierarchy System - UX Design Guide

## Overview

The folder system organizes mods in a hierarchical tree structure mirroring the actual file system. Mods can be placed in:
1. **Root folder** (`~mods`) - the main mods directory
2. **Subfolders** - physical subdirectories within `~mods`

---

## Data Structures

### ModFolder

Each folder (including root) has this structure:

```json
{
  "id": "~mods",              // Unique identifier (actual folder name)
  "name": "~mods",            // Display name (same as id)
  "enabled": true,            // Is folder enabled (affects all mods inside)
  "expanded": true,           // Is folder expanded in tree view
  "color": [255, 128, 0],     // Optional custom color [R, G, B]
  "depth": 0,                 // 0 = root, 1 = direct subfolder
  "parent_id": null,          // null for root, root folder name for subfolders
  "is_root": true,            // true only for the root folder
  "mod_count": 12             // Number of mods directly in this folder
}
```

### RootFolderInfo

Detailed root folder information:

```json
{
  "name": "~mods",                                    // Actual folder name
  "path": "C:\\Marvel Rivals\\~mods",                 // Full path
  "direct_mod_count": 5,                              // Mods directly in root
  "subfolder_count": 3                                // Number of subfolders
}
```

### ModEntry.folder_id

Each mod has a `folder_id` field:
- `"~mods"` (or actual root name) = mod is in the root folder
- `"SubfolderName"` = mod is in that subfolder

**Note:** All mods now have a folder_id matching their containing folder's actual name. No more `null` or `"_root"` values.

---

## Backend Commands

### `get_folders`

**Purpose:** Get all folders as a hierarchical list.

**Returns:** Array of `ModFolder` objects, root folder first:

```json
[
  { "id": "~mods", "name": "~mods", "depth": 0, "is_root": true, "mod_count": 5, ... },
  { "id": "Characters", "name": "Characters", "depth": 1, "parent_id": "~mods", "mod_count": 8, ... },
  { "id": "Maps", "name": "Maps", "depth": 1, "parent_id": "~mods", "mod_count": 3, ... },
  { "id": "UI", "name": "UI", "depth": 1, "parent_id": "~mods", "mod_count": 2, ... }
]
```

**UX Implementation:**
1. Display as indented tree based on `depth`
2. Root folder is always first and always visible
3. Show `mod_count` badge next to each folder
4. Use `is_root` to apply special styling to root

---

### `get_root_folder_info`

**Purpose:** Get detailed info about the root mods folder.

**Returns:** `RootFolderInfo` object

**UX Implementation:**
- Use for header display: "~mods (5 mods, 3 folders)"
- Show full path in settings or tooltips

---

### `create_folder`

**Purpose:** Create a new subfolder.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | ✅ | Folder name (will be directory name) |

**Returns:** The folder name on success

**UX Implementation:**
- Show "New Folder" button/icon
- Inline rename field after creation
- Validate: no special characters that break paths

---

### `update_folder`

**Purpose:** Update folder properties (color, expanded, enabled).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `folder` | ModFolder | ✅ | Updated folder object |

**UX Implementation:**
- Color picker for custom folder colors
- Toggle switches for enabled/expanded
- Save on change (no explicit save button)

---

### `delete_folder`

**Purpose:** Delete an empty folder.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | ✅ | Folder ID to delete |

**Returns:** Error if folder not empty

**UX Implementation:**
- Confirm dialog: "Delete folder '{name}'?"
- If not empty, show: "Move or delete mods first"
- Cannot delete root folder

---

### `assign_mod_to_folder`

**Purpose:** Move a mod to a different folder.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `mod_path` | string | ✅ | Current path of the mod |
| `folder_id` | string | ✅ | Target folder ID (use root folder name like `"~mods"` for root) |

**UX Implementation:**
- Drag & drop mods between folders
- Right-click context menu → "Move to" submenu
- Show all available folders in submenu

---

## Tree View Design Recommendation

```
📁 ~mods (5 mods)                    ← Root folder (always visible)
├── 📁 Characters (8 mods)           ← Subfolder (depth 1)
├── 📁 Maps (3 mods) 
└── 📁 UI (2 mods)
```

### Visual Hierarchy

| Element | Indent | Icon | Style |
|---------|--------|------|-------|
| Root folder | 0 | 📁 or custom | Bold, accent color |
| Subfolder | 16px | 📁 | Normal weight |

---

## Interaction Patterns

### Expanding/Collapsing
- Click folder row or chevron icon
- `expanded` property persists in state
- Animate with slide transition

### Drag & Drop
1. Drag mod → highlight valid drop targets
2. Drop on folder → calls `assign_mod_to_folder`
3. Drop on root → folder_id becomes root folder name (e.g., "~mods")

### Context Menu

Right-click on folder:
```
┌────────────────────────┐
│ Rename                 │
│ Change Color...        │
├────────────────────────┤
│ Enable All Mods        │
│ Disable All Mods       │
├────────────────────────┤
│ Delete Folder          │ ← Grayed if not empty
└────────────────────────┘
```

Right-click on mod:
```
┌────────────────────────┐
│ Enable/Disable         │
│ Delete                 │
├────────────────────────┤
│ Move to →             │→ ┌──────────────┐
│                        │  │ ~mods        │
│                        │  │ Characters   │
│                        │  │ Maps         │
│                        │  │ UI           │
│                        │  └──────────────┘
└────────────────────────┘
```

---

## Filter Integration

When filtering by folder:
- `"all"` → Show all mods (flatten tree or show tree)
- `"~mods"` (root name) → Show only mods directly in root
- `"Characters"` → Show only mods in Characters folder

The filtering uses `mod.folder_id` to match:
```javascript
// Root folder ID is the actual folder name (e.g., "~mods")
const rootFolder = folders.find(f => f.is_root);
if (selectedFolderId === rootFolder?.id) {
  // Show mods where folder_id === root folder name
} else {
  // Show mods where folder_id === selectedFolderId
}
```

---

## Migration Notes (For Frontend)

### Breaking Change

Previously, mods directly in `~mods` had `folder_id: null`.

Now, they have `folder_id` set to the actual root folder name (e.g., `"~mods"`).

**Update filter logic:**
```javascript
// Old
if (mod.folder_id === null) { /* root mod */ }

// New - use is_root flag from folders to identify root  
const rootFolder = folders.find(f => f.is_root);
if (mod.folder_id === rootFolder?.id) { /* root mod */ }
```

### New Fields

`ModFolder` now includes:
- `depth` - for indentation (0 or 1)
- `parent_id` - for tree relationships
- `is_root` - for root folder identification
- `mod_count` - for badge display

---

## Accessibility

1. **Keyboard Navigation:**
   - Arrow keys to navigate tree
   - Enter to expand/collapse
   - Space to toggle mod/folder enabled

2. **Screen Readers:**
   - Announce "folder" or "mod file"
   - Announce mod count: "Characters folder, 8 mods"
   - Announce expanded/collapsed state

3. **Focus Indicators:**
   - Clear focus ring on tree items
   - Focus trap within context menus

---

## Future Enhancements

1. **Nested Subfolders:** Backend already supports `depth` for deeper nesting
2. **Folder Icons:** Custom icons per folder
3. **Bulk Operations:** Select multiple mods for batch move
4. **Search Within Folder:** Filter mods by folder + search term
5. **Folder Statistics:** Show total size, mod types breakdown
