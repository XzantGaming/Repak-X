# Frontend Implementation: Replace "Ungrouped" with Root Folder Name

## Issue

The sidebar currently shows a hardcoded "Ungrouped" folder that is separate from the actual root folder (`~mods`). This creates confusion:

```
Folders
├── All Mods (12)
├── Ungrouped (0)     ← PROBLEM: Hardcoded, shows 0 mods
├── ~mods (3)         ← The actual root folder with mods
├── DMC (7)
└── Valox (2)
```

## Desired Behavior

```
Folders
├── All Mods (12)
├── ~mods (3)         ← Root folder (dynamically named)
├── DMC (7)
└── Valox (2)
```

The root folder should:
1. Use the actual folder name from the backend (e.g., `~mods`)
2. Display mods that are directly in the root folder
3. Be identified by `is_root: true` flag from backend

---

## Backend Changes (Already Implemented)

The backend now returns:
- Root folder with `is_root: true` flag
- All mods have `folder_id` set to actual folder names (not `null`)
- Root mods have `folder_id` matching the root folder's `id`

**Example `get_folders` response:**
```json
[
  { "id": "~mods", "name": "~mods", "is_root": true, "mod_count": 3, "depth": 0 },
  { "id": "DMC", "name": "DMC", "is_root": false, "mod_count": 7, "depth": 1 },
  { "id": "Valox", "name": "Valox", "is_root": false, "mod_count": 2, "depth": 1 }
]
```

**Mods in root now have:**
```json
{ "folder_id": "~mods", ... }  // NOT null anymore
```

---

## Required Frontend Changes (App.jsx)

### 1. Remove Hardcoded "Ungrouped" Folder Entry

**Location:** Around line 886-893

**Current Code:**
```jsx
<div 
  className={`folder-item ${selectedFolderId === 'ungrouped' ? 'active' : ''} ${mods.filter(m => !m.folder_id).length === 0 ? 'empty' : ''}`}
  onClick={() => setSelectedFolderId('ungrouped')}
>
  <FolderIcon fontSize="small" />
  <span className="folder-name">Ungrouped</span>
  <span className="folder-count">{mods.filter(m => !m.folder_id).length}</span>
</div>
```

**Action:** DELETE this entire `<div>` block.

---

### 2. Update Folder Filter Logic

**Location:** Around line 658-664

**Current Code:**
```jsx
const filteredMods = mods.filter(mod => {
  // Folder filter
  if (selectedFolderId !== 'all') {
    if (selectedFolderId === 'ungrouped') {
      if (mod.folder_id) return false
    } else {
      if (mod.folder_id !== selectedFolderId) return false
    }
  }
```

**New Code:**
```jsx
const filteredMods = mods.filter(mod => {
  // Folder filter
  if (selectedFolderId !== 'all') {
    // All folders (including root) now use folder_id matching
    if (mod.folder_id !== selectedFolderId) return false
  }
```

---

### 3. Update Header Title Display

**Location:** Around line 923-927

**Current Code:**
```jsx
<h2>
  {selectedFolderId === 'all' ? 'All Mods' : 
   selectedFolderId === 'ungrouped' ? 'Ungrouped Mods' : 
   folders.find(f => f.id === selectedFolderId)?.name || 'Unknown Folder'}
</h2>
```

**New Code:**
```jsx
<h2>
  {selectedFolderId === 'all' ? 'All Mods' : 
   folders.find(f => f.id === selectedFolderId)?.name || 'Unknown Folder'}
</h2>
```

---

### 4. Ensure Root Folder is Displayed First in List

**Location:** Where folders are mapped (around line 894)

The backend already returns the root folder first. If you want to ensure root folder appears at the top with special styling:

```jsx
{folders.map(folder => (
  <div 
    key={folder.id}
    className={`folder-item ${selectedFolderId === folder.id ? 'active' : ''} ${folder.is_root ? 'root-folder' : ''}`}
    onClick={() => setSelectedFolderId(folder.id)}
  >
    <FolderIcon fontSize="small" />
    <span className="folder-name">{folder.name}</span>
    <span className="folder-count">{folder.mod_count}</span>
  </div>
))}
```

---

### 5. Optional: Add Root Folder Styling (App.css)

```css
.folder-item.root-folder {
  font-weight: 600;
  /* Optional: different icon or color */
}

.folder-item.root-folder .folder-icon {
  color: var(--accent-color);
}
```

---

## Summary of Changes

| File | Line(s) | Action |
|------|---------|--------|
| `App.jsx` | ~886-893 | DELETE hardcoded "Ungrouped" div |
| `App.jsx` | ~661-662 | REMOVE `ungrouped` special case in filter |
| `App.jsx` | ~926 | REMOVE `ungrouped` case in header title |
| `App.css` | (optional) | ADD `.root-folder` styling |

---

## Testing Checklist

- [ ] "Ungrouped" no longer appears in folder list
- [ ] Root folder shows with actual name (e.g., `~mods`)
- [ ] Root folder shows correct mod count
- [ ] Clicking root folder shows mods in that folder
- [ ] Subfolders still work correctly
- [ ] "All Mods" still shows all mods
- [ ] Mod details show correct folder name (not `_root`)

---

## Context Menu Update (Optional)

If there's a context menu for "Move to folder", update to use root folder name:

**Location:** `components/ContextMenu.jsx` around line 76

**Current:**
```jsx
<div className="context-menu-item" onClick={() => { onMoveTo(null); onClose(); }}>
  Ungrouped
</div>
```

**New:**
```jsx
{folders.filter(f => f.is_root).map(rootFolder => (
  <div key={rootFolder.id} className="context-menu-item" onClick={() => { onMoveTo(rootFolder.id); onClose(); }}>
    {rootFolder.name}
  </div>
))}
```

Or simpler:
```jsx
<div className="context-menu-item" onClick={() => { 
  const rootFolder = folders.find(f => f.is_root);
  onMoveTo(rootFolder?.id || null); 
  onClose(); 
}}>
  {folders.find(f => f.is_root)?.name || 'Root'}
</div>
```
