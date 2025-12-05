# Mod Details Panel Error Persistence Bug

## Problem Description

When a user selects a mod, the details viewer appears on the right side. If the user then deletes that mod, an error message appears: "Error Loading Details - Mod file does not exist: [path]". This is expected behavior.

However, when the user selects a different mod after deletion, the error message persists and the mod details viewer doesn't update with the new mod's information. Only the mod name above the details panel updates, but the error message remains visible.

## Root Cause

The issue occurs due to **stale error state** in the `ModDetailsPanel` component. There are two contributing factors:

### 1. Error State Not Cleared in ModDetailsPanel Component

**Location:** `repak-gui/src/components/ModDetailsPanel.jsx` (lines 12-25)

```jsx
useEffect(() => {
  if (mod) {
    if (initialDetails && initialDetails.mod_path === mod.path) {
      setDetails(initialDetails)
      setLoading(false)
    } else if (initialDetails && !initialDetails.mod_path) {
      // Fallback if mod_path isn't in details (older backend version?)
      setDetails(initialDetails)
      setLoading(false)
    } else {
      loadModDetails()
    }
  }
}, [mod, initialDetails])
```

**Problem:** When the `mod` prop changes (user selects a new mod), the `error` state is never reset. The error from the previous deleted mod persists even though we're now looking at a different mod.

**Why it happens:**
1. User selects Mod A → Details load successfully
2. User deletes Mod A → Error is set: "Mod file does not exist"
3. User selects Mod B → The `mod` prop changes, but `setError(null)` is never called
4. The component may use cached `initialDetails` for Mod B (lines 14-20), which bypasses the `loadModDetails()` function
5. Even if `loadModDetails()` is called, it sets `setError(null)` at line 60, but only AFTER starting to load
6. The error state from Mod A remains visible while Mod B's details are being fetched

### 2. Selected Mod Not Cleared on Deletion

**Location:** `repak-gui/src/App.jsx` (lines 577-591)

```jsx
const handleDeleteMod = async (modPath) => {
  if (gameRunning) {
    setStatus('Cannot delete mods while game is running')
    return
  }
  
  try {
    await invoke('delete_mod', { path: modPath })
    setStatus('Mod deleted')
    await loadMods()
  } catch (error) {
    setStatus('Error deleting mod: ' + error)
  }
}
```

**Problem:** When a mod is deleted, the `selectedMod` state is not cleared. This means the `ModDetailsPanel` component continues to receive the deleted mod as a prop, causing it to attempt loading details for a non-existent file.

## Solutions

### Solution 1: Reset Error State in ModDetailsPanel (Primary Fix)

Add `setError(null)` at the beginning of the useEffect when the mod changes:

```jsx
useEffect(() => {
  if (mod) {
    // Reset error state when mod changes
    setError(null)
    
    if (initialDetails && initialDetails.mod_path === mod.path) {
      setDetails(initialDetails)
      setLoading(false)
    } else if (initialDetails && !initialDetails.mod_path) {
      setDetails(initialDetails)
      setLoading(false)
    } else {
      loadModDetails()
    }
  }
}, [mod, initialDetails])
```

This ensures that whenever a new mod is selected, any previous error state is cleared before attempting to load or display the new mod's details.

### Solution 2: Clear Selected Mod on Deletion (Secondary Fix)

Clear the `selectedMod` state when deleting the currently selected mod:

```jsx
const handleDeleteMod = async (modPath) => {
  if (gameRunning) {
    setStatus('Cannot delete mods while game is running')
    return
  }
  
  try {
    await invoke('delete_mod', { path: modPath })
    setStatus('Mod deleted')
    
    // Clear selection if the deleted mod was selected
    if (selectedMod && selectedMod.path === modPath) {
      setSelectedMod(null)
    }
    
    await loadMods()
  } catch (error) {
    setStatus('Error deleting mod: ' + error)
  }
}
```

This prevents the details panel from trying to display information about a mod that no longer exists.

## Recommended Approach

Implement **both solutions** for a robust fix:

1. **Solution 1** ensures the error state is always cleared when switching between mods
2. **Solution 2** ensures the UI doesn't try to show details for a deleted mod in the first place

Together, these changes will:
- Prevent error messages from persisting when selecting a new mod
- Clear the details panel immediately when a selected mod is deleted
- Provide a better user experience with proper state management

## Testing Steps

After implementing the fixes:

1. Select a mod → Verify details appear correctly
2. Delete the selected mod → Verify error message appears
3. Select a different mod → Verify error message disappears and new mod details load correctly
4. Delete a mod that isn't selected → Verify the currently selected mod's details remain visible
