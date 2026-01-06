# UX Specification: Mod Conflict Detection Context Menu

## Overview

Add a context menu option to check if a specific mod has conflicts with other installed mods. This allows users to quickly identify which mods are modifying the same game files.

---

## Backend API

### Tauri Command

```javascript
invoke('check_single_mod_conflicts', { modPath: string })
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `modPath` | `string` | Full filesystem path to the mod's `.pak` file |

### Response Type

```typescript
interface SingleModConflict {
  conflicting_mod_path: string;    // Full path to the conflicting mod
  conflicting_mod_name: string;    // Filename of the conflicting mod
  overlapping_files: string[];     // List of files both mods modify
  priority_comparison: string;     // Human-readable priority comparison
  affected_characters: string[];   // Character IDs affected (e.g., ["1050", "1033"])
}

// Returns: SingleModConflict[]
```

### Example Response

```json
[
  {
    "conflicting_mod_path": "C:/Games/MarvelRivals/~mods/LokiSkin_9999999_P.pak",
    "conflicting_mod_name": "LokiSkin_9999999_P.pak",
    "overlapping_files": [
      "Marvel/Content/Characters/1050/1050800/Meshes/SK_1050800.uasset",
      "Marvel/Content/Characters/1050/1050800/Textures/T_1050800_Body_D.uasset"
    ],
    "priority_comparison": "Same priority (conflict!)",
    "affected_characters": ["1050"]
  }
]
```

---

## UI Implementation

### 1. Context Menu Item

Add a new item to the mod context menu (right-click menu):

```
┌─────────────────────────────┐
│ Enable/Disable              │
│ Set Priority...             │
│ ─────────────────────────── │
│ Check for Conflicts         │  ← NEW ITEM
│ ─────────────────────────── │
│ Open in Explorer            │
│ Delete                      │
└─────────────────────────────┘
```

**Menu Item Properties:**
- **Label:** "Check for Conflicts"
- **Icon:** Suggested: `AlertTriangle` or `Search` from Lucide icons
- **Shortcut:** None (optional: `Ctrl+K`)

### 2. Loading State

While checking conflicts, show a loading indicator:

```
┌─────────────────────────────────────────┐
│  ⟳ Checking for conflicts...           │
└─────────────────────────────────────────┘
```

**Behavior:**
- Disable the context menu item while loading
- Show a toast or inline spinner
- Typical operation takes 1-5 seconds depending on number of installed mods

### 3. Results Display

#### Option A: Modal Dialog (Recommended)

```
┌─────────────────────────────────────────────────────────────┐
│  Conflict Check: MyModName_9999999_P.pak                  X │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ⚠ Found 2 conflicts                                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ LokiSkin_9999999_P.pak                              │   │
│  │ ─────────────────────────────────────────────────── │   │
│  │ Priority: Same priority (conflict!)                 │   │
│  │ Characters: Loki (1050)                             │   │
│  │ Overlapping files: 3                                │   │
│  │ [View Files ▼]                                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ AnotherMod_99999999_P.pak                           │   │
│  │ ─────────────────────────────────────────────────── │   │
│  │ Priority: Target has higher priority (1 vs 2)       │   │
│  │ Characters: Thor (1033)                             │   │
│  │ Overlapping files: 1                                │   │
│  │ [View Files ▼]                                      │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│                                          [Close]            │
└─────────────────────────────────────────────────────────────┘
```

#### Option B: Toast Notification (For No Conflicts)

```
┌─────────────────────────────────────────┐
│ ✓ No conflicts found                    │
│   MyModName_9999999_P.pak               │
└─────────────────────────────────────────┘
```

### 4. Expanded File List View

When user clicks "View Files":

```
┌─────────────────────────────────────────────────────────────┐
│ LokiSkin_9999999_P.pak                                      │
│ ─────────────────────────────────────────────────────────── │
│ Priority: Same priority (conflict!)                         │
│ Characters: Loki (1050)                                     │
│                                                             │
│ Overlapping files (3):                          [Hide ▲]   │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ • Marvel/Content/Characters/1050/1050800/Meshes/        │ │
│ │   SK_1050800.uasset                                     │ │
│ │ • Marvel/Content/Characters/1050/1050800/Textures/      │ │
│ │   T_1050800_Body_D.uasset                               │ │
│ │ • Marvel/Content/Characters/1050/1050800/Textures/      │ │
│ │   T_1050800_Body_N.uasset                               │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Color Coding

| Condition | Color | Meaning |
|-----------|-------|---------|
| Same priority | `#EF4444` (red) | True conflict - unpredictable behavior |
| Target higher priority | `#22C55E` (green) | Target mod will win |
| Target lower priority | `#F59E0B` (amber) | Other mod will win |

---

## Example React Implementation Skeleton

```jsx
// In your context menu handler
const handleCheckConflicts = async (mod) => {
  setIsCheckingConflicts(true);
  try {
    const conflicts = await invoke('check_single_mod_conflicts', { 
      modPath: mod.path 
    });
    
    if (conflicts.length === 0) {
      // Show success toast
      showToast({
        type: 'success',
        title: 'No conflicts found',
        description: mod.name
      });
    } else {
      // Open conflict modal
      setConflictResults({
        modName: mod.name,
        conflicts: conflicts
      });
      setShowConflictModal(true);
    }
  } catch (error) {
    showToast({
      type: 'error',
      title: 'Failed to check conflicts',
      description: error.message
    });
  } finally {
    setIsCheckingConflicts(false);
  }
};
```

---

## Accessibility

- Modal should trap focus
- Close on `Escape` key
- Screen reader announcements for conflict count
- High contrast colors for priority indicators

---

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Mod file doesn't exist | Show error toast: "Mod file not found" |
| Game path not set | Show error toast: "Please set game path first" |
| Disabled mods | Still check against disabled mods (they may be re-enabled) |
| IoStore mods (.utoc/.ucas) | Handled automatically by backend |
| Large number of conflicts (>10) | Show scrollable list with count badge |

---

## Related Components

- `ModListItem.jsx` - Add context menu trigger
- `ModDetailsPanel.jsx` - Alternative location for conflict check button
- Existing `check_mod_clashes` command - Checks ALL mods (global view)

---

## Notes for Implementation

1. The `modPath` must be the full filesystem path (e.g., `C:/Games/MarvelRivals/~mods/MyMod.pak`)
2. Character IDs can be mapped to names using the existing `character_data.json`
3. Priority comparison text is human-readable and can be displayed as-is
4. The backend handles both regular PAK files and IoStore packages automatically
