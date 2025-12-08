# Marvel Rivals Crash Detection - UI Implementation Guide

## Overview

This document provides a complete reference for implementing the crash detection UI in the React frontend. The backend crash monitoring system is fully implemented and tested - this guide will help you integrate it into the user interface.

---

## 🎯 Feature Summary

**What it does:**
- Monitors Marvel Rivals game process in real-time
- Detects crashes vs normal exits by checking for new crash folders
- Only reports crashes from the current game session (ignores old crashes)
- Provides detailed crash information including error messages, time in game, and active mods
- Logs all crash details to `Logs/repak-gui.log`

**What it prevents:**
- False positives from old crash folders
- Reporting crashes from previous sessions
- Duplicate crash notifications

---

## 📡 Available Tauri Commands

### 1. `monitor_game_for_crashes()`

**Primary crash detection command - call this periodically (every 2-5 seconds)**

```typescript
import { invoke } from '@tauri-apps/api/core';

// Returns crash info only when a crash is detected
const crashInfo = await invoke<CrashInfo | null>('monitor_game_for_crashes');

if (crashInfo) {
  // Show crash dialog
  showCrashNotification(crashInfo);
}
```

**Return Type:**
```typescript
interface CrashInfo {
  crash_folder: string;           // Path to crash folder
  timestamp: number;               // Unix timestamp
  error_message: string | null;   // e.g., "EXCEPTION_ACCESS_VIOLATION"
  crash_type: string | null;      // e.g., "Crash", "Assert", etc.
  seconds_since_start: number | null;  // Time in game before crash
  process_id: number | null;      // Game process ID
  enabled_mods: string[];         // List of enabled mod filenames
}
```

**Behavior:**
- Returns `null` most of the time (no crash detected)
- Returns `CrashInfo` object only when:
  1. Game was running (session started)
  2. Game stopped (process ended)
  3. New crash folder was created during that session
- Automatically resets after each game session

---

### 2. `get_crash_history()`

**Get list of all crash folders (for crash history view)**

```typescript
const crashFolders = await invoke<string[]>('get_crash_history');
// Returns array of crash folder paths, sorted newest first
// Example: ["C:\\Users\\...\\Crashes\\UECC-Windows-..._0000", ...]
```

---

### 3. `get_total_crashes()`

**Get total count of crash folders**

```typescript
const totalCrashes = await invoke<number>('get_total_crashes');
// Returns: 170 (for example)
```

---

### 4. `clear_crash_logs()`

**Delete all crash folders (cleanup function)**

```typescript
const deletedCount = await invoke<number>('clear_crash_logs');
// Returns number of crash folders deleted
```

---

### 5. `get_crash_log_path()`

**Get path to crash directory**

```typescript
const crashPath = await invoke<string>('get_crash_log_path');
// Returns: "C:\\Users\\YourName\\AppData\\Local\\Marvel\\Saved\\Crashes"
```

---

### 6. `dismiss_crash_dialog()`

**No-op command (frontend handles dialog state)**

```typescript
await invoke('dismiss_crash_dialog');
// Just for consistency - you can manage dialog state locally
```

---

## 🎨 Recommended UI Implementation

### Option 1: Simple Toast Notification

```typescript
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

function CrashMonitor() {
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const crashInfo = await invoke<CrashInfo | null>('monitor_game_for_crashes');
        
        if (crashInfo) {
          // Show toast notification
          toast.error('Game Crashed!', {
            description: `Error: ${crashInfo.error_message || 'Unknown'}`,
            action: {
              label: 'View Details',
              onClick: () => showCrashDetails(crashInfo)
            }
          });
        }
      } catch (error) {
        console.error('Crash monitoring error:', error);
      }
    }, 3000); // Check every 3 seconds
    
    return () => clearInterval(interval);
  }, []);
  
  return null; // This is a background monitor
}
```

---

### Option 2: Modal Dialog with Details

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

function CrashDetectionSystem() {
  const [crashInfo, setCrashInfo] = useState<CrashInfo | null>(null);
  const [showDialog, setShowDialog] = useState(false);
  
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const crash = await invoke<CrashInfo | null>('monitor_game_for_crashes');
        
        if (crash) {
          setCrashInfo(crash);
          setShowDialog(true);
        }
      } catch (error) {
        console.error('Crash monitoring error:', error);
      }
    }, 3000);
    
    return () => clearInterval(interval);
  }, []);
  
  const formatTime = (seconds: number | null) => {
    if (!seconds) return 'Unknown';
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}m ${secs}s`;
  };
  
  return (
    <>
      {showDialog && crashInfo && (
        <div className="crash-dialog-overlay">
          <div className="crash-dialog">
            <div className="crash-header">
              <h2>⚠️ Game Crash Detected</h2>
            </div>
            
            <div className="crash-body">
              <div className="crash-detail">
                <strong>Error:</strong>
                <span>{crashInfo.error_message || 'Unknown error'}</span>
              </div>
              
              <div className="crash-detail">
                <strong>Type:</strong>
                <span>{crashInfo.crash_type || 'Unknown'}</span>
              </div>
              
              <div className="crash-detail">
                <strong>Time in game:</strong>
                <span>{formatTime(crashInfo.seconds_since_start)}</span>
              </div>
              
              {crashInfo.enabled_mods.length > 0 && (
                <div className="crash-detail">
                  <strong>Active mods ({crashInfo.enabled_mods.length}):</strong>
                  <ul className="mod-list">
                    {crashInfo.enabled_mods.map((mod, i) => (
                      <li key={i}>{mod}</li>
                    ))}
                  </ul>
                </div>
              )}
              
              <div className="crash-detail">
                <strong>Crash folder:</strong>
                <code>{crashInfo.crash_folder}</code>
              </div>
            </div>
            
            <div className="crash-footer">
              <button onClick={() => setShowDialog(false)}>
                Dismiss
              </button>
              <button onClick={() => {
                // Open crash folder in explorer
                invoke('open_path', { path: crashInfo.crash_folder });
              }}>
                Open Crash Folder
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
```

---

### Option 3: Settings Page - Crash History

```typescript
function CrashHistorySettings() {
  const [crashes, setCrashes] = useState<string[]>([]);
  const [totalCrashes, setTotalCrashes] = useState(0);
  const [loading, setLoading] = useState(false);
  
  useEffect(() => {
    loadCrashHistory();
  }, []);
  
  const loadCrashHistory = async () => {
    setLoading(true);
    try {
      const [history, total] = await Promise.all([
        invoke<string[]>('get_crash_history'),
        invoke<number>('get_total_crashes')
      ]);
      setCrashes(history);
      setTotalCrashes(total);
    } catch (error) {
      console.error('Failed to load crash history:', error);
    } finally {
      setLoading(false);
    }
  };
  
  const clearAllCrashes = async () => {
    if (!confirm(`Delete all ${totalCrashes} crash logs?`)) return;
    
    try {
      const deleted = await invoke<number>('clear_crash_logs');
      toast.success(`Deleted ${deleted} crash logs`);
      loadCrashHistory();
    } catch (error) {
      toast.error('Failed to clear crash logs');
    }
  };
  
  return (
    <div className="crash-history-section">
      <h3>Crash History</h3>
      
      <div className="crash-stats">
        <p>Total crashes recorded: <strong>{totalCrashes}</strong></p>
        <button onClick={clearAllCrashes} disabled={totalCrashes === 0}>
          Clear All Crash Logs
        </button>
      </div>
      
      {loading ? (
        <p>Loading crash history...</p>
      ) : (
        <div className="crash-list">
          {crashes.length === 0 ? (
            <p>No crashes recorded</p>
          ) : (
            <ul>
              {crashes.slice(0, 10).map((crash, i) => (
                <li key={i}>
                  <code>{crash}</code>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
```

---

## 🔧 Integration Checklist

- [ ] Add `CrashMonitor` component to main app layout
- [ ] Set up periodic polling (3-5 second interval)
- [ ] Create crash notification UI (toast/modal/banner)
- [ ] Display crash details (error, time, mods)
- [ ] Add crash history view in settings
- [ ] Add "Clear Crash Logs" button
- [ ] Test with actual game crash
- [ ] Test that old crashes are ignored
- [ ] Verify logs are written to `Logs/repak-gui.log`

---

## 📝 Example Log Output

When a crash is detected, you'll see this in `Logs/repak-gui.log`:

```
[ERROR] ⚠️ ═══════════════════════════════════════════════════════════════
[ERROR] ⚠️ CRASH DETECTED! Marvel Rivals crashed during this session!
[ERROR] ⚠️ ═══════════════════════════════════════════════════════════════
[ERROR] ⚠️ Found 1 crash folder(s) from this session
[ERROR] ⚠️ Crash Details:
[ERROR] ⚠️   Error: EXCEPTION_ACCESS_VIOLATION reading address 0x0000000000000000
[ERROR] ⚠️   Type: Crash
[ERROR] ⚠️   Time in game: 15m 32s
[ERROR] ⚠️   Crash folder: "C:\\Users\\...\\Crashes\\UECC-Windows-..._0000"
[ERROR] ⚠️   Mods enabled: 3 mod(s)
[ERROR] ⚠️   Active mods:
[ERROR] ⚠️     - SpiderManMod.pak
[ERROR] ⚠️     - IronManSkin.pak
[ERROR] ⚠️     - CustomUI.pak
[ERROR] ⚠️ ═══════════════════════════════════════════════════════════════
```

When game closes normally:

```
[INFO] ✓ ═══════════════════════════════════════════════════════════════
[INFO] ✓ Game closed normally - no crashes detected this session
[INFO] ✓ ═══════════════════════════════════════════════════════════════
```

---

## 🧪 Testing Guide

### Test Case 1: Normal Exit (No Crash)
1. Start the app
2. Launch Marvel Rivals
3. Play for a bit
4. Close game normally
5. **Expected:** Log shows "Game closed normally"
6. **Expected:** No crash notification appears

### Test Case 2: Actual Crash
1. Start the app
2. Launch Marvel Rivals
3. Trigger a crash (use a known crashy mod)
4. **Expected:** Log shows crash details with error message
5. **Expected:** Crash notification appears in UI

### Test Case 3: Old Crash (Should Ignore)
1. Have existing crash folders in `%LOCALAPPDATA%\Marvel\Saved\Crashes`
2. Start the app (game not running)
3. Launch game, play, close normally
4. **Expected:** Old crashes are ignored
5. **Expected:** No crash notification

### Test Case 4: Multiple Sessions
1. Start app → Launch game → Crash → See notification
2. Fix mod → Launch game again → Close normally
3. **Expected:** Second session shows "closed normally"
4. **Expected:** First crash is NOT reported again

---

## 🎨 Suggested UI Styling

```css
.crash-dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.crash-dialog {
  background: #1a1a1a;
  border: 2px solid #ff4444;
  border-radius: 8px;
  padding: 24px;
  max-width: 600px;
  width: 90%;
  color: #fff;
}

.crash-header {
  border-bottom: 1px solid #333;
  padding-bottom: 12px;
  margin-bottom: 16px;
}

.crash-header h2 {
  margin: 0;
  color: #ff4444;
  font-size: 20px;
}

.crash-detail {
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.crash-detail strong {
  color: #888;
  font-size: 12px;
  text-transform: uppercase;
}

.crash-detail span,
.crash-detail code {
  color: #fff;
  font-size: 14px;
}

.mod-list {
  list-style: none;
  padding: 8px 0 0 0;
  margin: 0;
}

.mod-list li {
  padding: 4px 8px;
  background: #2a2a2a;
  margin-bottom: 4px;
  border-radius: 4px;
  font-family: monospace;
}

.crash-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid #333;
}

.crash-footer button {
  padding: 8px 16px;
  border-radius: 4px;
  border: none;
  cursor: pointer;
  font-size: 14px;
}

.crash-footer button:first-child {
  background: #333;
  color: #fff;
}

.crash-footer button:last-child {
  background: #ff4444;
  color: #fff;
}
```

---

## 🚀 Quick Start Example

**Minimal implementation to get started:**

```typescript
// Add to your main App.tsx or layout component

import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

function App() {
  useEffect(() => {
    // Start crash monitoring
    const interval = setInterval(async () => {
      const crash = await invoke('monitor_game_for_crashes');
      if (crash) {
        alert(`Game crashed! Error: ${crash.error_message}`);
      }
    }, 3000);
    
    return () => clearInterval(interval);
  }, []);
  
  return (
    <div className="app">
      {/* Your existing app content */}
    </div>
  );
}
```

---

## 📞 Support & Questions

- Check `Logs/repak-gui.log` for detailed crash information
- Crash folders are in: `%LOCALAPPDATA%\Marvel\Saved\Crashes`
- Backend is fully implemented and tested
- All Tauri commands are registered and working

**Backend Files:**
- `src/crash_monitor.rs` - Core crash detection logic
- `src/main_tauri.rs` - Tauri command implementations (lines 1791-1931)
- `src/app_state.rs` - State management (for egui version)

---

## ✅ Implementation Complete

The backend crash detection system is **100% complete and tested**. This document provides everything needed to build the UI. Good luck! 🚀
