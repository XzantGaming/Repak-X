# Mod Details Panel Race Condition Fix

## Problem
In release builds, some mods show "Loading mod details..." indefinitely while working fine in dev mode. The affected mods appear random.

## Root Cause
There's a **race condition** in `ModDetailsPanel.jsx`. The `useEffect` hook calls `loadModDetails()` which is an async function, but there's no cancellation mechanism when the `mod` prop changes.

In release builds, the optimized code runs faster, exposing timing issues:
1. User clicks Mod A → `loadModDetails()` starts for Mod A
2. User quickly clicks Mod B → component re-renders, new `loadModDetails()` starts for Mod B  
3. Mod A's response arrives first → sets `details` and `loading=false`
4. Mod B's response arrives → but the component already shows Mod A's details
5. OR: Mod B's request never completes properly due to stale closure

## Fix Required

In `src/components/ModDetailsPanel.jsx`, update the `useEffect` to use an abort flag:

### Current Code (lines 16-30):
```jsx
useEffect(() => {
  if (mod) {
    setError(null) // Reset error state when mod changes
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

### Fixed Code:
```jsx
useEffect(() => {
  let cancelled = false
  
  const loadDetails = async () => {
    if (!mod) return
    
    setError(null)
    
    // Check if we already have details for this mod
    if (initialDetails && (initialDetails.mod_path === mod.path || !initialDetails.mod_path)) {
      setDetails(initialDetails)
      setLoading(false)
      return
    }
    
    // Need to fetch details
    try {
      setLoading(true)
      setError(null)
      console.log('Loading details for:', mod.path)
      const modDetails = await invoke('get_mod_details', {
        modPath: mod.path
      })
      
      // Check if this request is still relevant
      if (cancelled) {
        console.log('Request cancelled, ignoring result for:', mod.path)
        return
      }
      
      console.log('Received details:', modDetails)
      setDetails(modDetails)
    } catch (err) {
      if (cancelled) return
      console.error('Failed to load mod details:', err)
      setError(err.toString())
    } finally {
      if (!cancelled) {
        setLoading(false)
      }
    }
  }
  
  loadDetails()
  
  // Cleanup: cancel pending request when mod changes
  return () => {
    cancelled = true
  }
}, [mod, initialDetails])
```

### Also Remove the Standalone `loadModDetails` Function (lines 59-75):
The function is now inlined in the useEffect, so remove:
```jsx
const loadModDetails = async () => {
  try {
    setLoading(true)
    setError(null)
    console.log('Loading details for:', mod.path)
    const modDetails = await invoke('get_mod_details', {
      modPath: mod.path
    })
    console.log('Received details:', modDetails)
    setDetails(modDetails)
  } catch (err) {
    console.error('Failed to load mod details:', err)
    setError(err.toString())
  } finally {
    setLoading(false)
  }
}
```

## Additional Rust Fix Applied
The `read_utoc` function in `src/utoc_utils.rs` was also updated to not panic on parse failures. It now returns an empty `Vec` and logs a warning instead of crashing the async command.

This prevents the async Tauri command from hanging forever if a `.utoc` file can't be parsed.
