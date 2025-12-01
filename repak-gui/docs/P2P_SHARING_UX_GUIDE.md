# P2P Mod Sharing - UX Design Guide

## Overview

The P2P (Peer-to-Peer) Mod Sharing system allows users to securely share mod packs directly with other users without needing external file hosting services. All transfers are encrypted end-to-end.

---

## User Flow Diagrams

### Sharing Flow (Host)
```
┌─────────────────────────────────────────────────────────────────────┐
│  1. SELECT MODS  →  2. CONFIGURE PACK  →  3. START SHARING  →  4. SHARE CODE  │
└─────────────────────────────────────────────────────────────────────┘
```

### Receiving Flow (Client)
```
┌─────────────────────────────────────────────────────────────────────┐
│  1. ENTER CODE  →  2. PREVIEW PACK  →  3. CONFIRM DOWNLOAD  →  4. TRANSFER  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Backend Commands Reference

### Sharing Commands (Host Side)

#### `p2p_start_sharing`
**Purpose:** Start hosting a mod pack for sharing.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | ✅ | Display name for the mod pack (e.g., "My Iron Man Collection") |
| `description` | string | ✅ | Description of what's included |
| `mod_paths` | string[] | ✅ | Array of full file paths to .pak files |
| `creator` | string | ❌ | Optional creator name/handle |

**Returns:** `ShareSession` object containing:
```json
{
  "share_code": "A3B7-K9M2-X5P8",      // User-friendly code to share
  "encryption_key": "base64string...", // Internal - don't display
  "local_ip": "192.168.1.100",         // Host's local IP
  "port": 47820,                        // Port being used
  "connection_string": "full_string",  // Full code (for copy/paste)
  "active": true                        // Session is active
}
```

**UX Implementation:**
1. Show a modal/dialog for creating a share pack
2. Allow multi-select of installed mods
3. Provide name and description inputs
4. On success, display the `share_code` prominently with a COPY button
5. Show connection status indicator (green dot = active)

---

#### `p2p_stop_sharing`
**Purpose:** Stop hosting and close the share session.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| (none) | - | - | - |

**Returns:** `void`

**UX Implementation:**
- Provide a clear "Stop Sharing" button when session is active
- Confirm before stopping (users may have active downloads)
- Update status indicator to inactive state

---

#### `p2p_get_share_session`
**Purpose:** Get current share session info (for status display).

**Returns:** `ShareSession | null`

**UX Implementation:**
- Use for persistent status bar display
- Show share code if active
- Show number of connected peers (future enhancement)

---

#### `p2p_is_sharing`
**Purpose:** Quick check if currently hosting.

**Returns:** `boolean`

**UX Implementation:**
- Use to conditionally render "Start Sharing" vs "Stop Sharing" buttons
- Control visibility of share panel

---

### Receiving Commands (Client Side)

#### `p2p_start_receiving`
**Purpose:** Connect to a host and start downloading mods.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `connection_string` | string | ✅ | Full connection string from host |
| `client_name` | string | ❌ | Optional name to identify to host |

**Returns:** `void` (progress tracked via `p2p_get_receive_progress`)

**UX Implementation:**
1. Show input field for paste/type connection string
2. Validate with `p2p_validate_connection_string` before connecting
3. Show connecting spinner
4. Switch to progress view on success

---

#### `p2p_stop_receiving`
**Purpose:** Cancel an in-progress download.

**Returns:** `void`

**UX Implementation:**
- Confirm before cancelling (partial files will be deleted)
- Update UI to show cancelled state

---

#### `p2p_get_receive_progress`
**Purpose:** Get current transfer progress for UI updates.

**Returns:** `TransferProgress` object:
```json
{
  "current_file": "CoolSkin_9999999_P.pak",
  "files_completed": 2,
  "total_files": 5,
  "bytes_transferred": 52428800,    // 50MB
  "total_bytes": 104857600,          // 100MB
  "status": "Transferring"           // See status enum below
}
```

**Status Values:**
| Status | Description | UX Action |
|--------|-------------|-----------|
| `Connecting` | Establishing connection | Show spinner + "Connecting..." |
| `Handshaking` | Exchanging pack info | Show "Verifying..." |
| `Transferring` | Active file transfer | Show progress bar |
| `Verifying` | Checking file integrity | Show "Verifying..." |
| `Completed` | All done! | Show success + refresh mod list |
| `Failed` | Error occurred (includes message) | Show error message |
| `Cancelled` | User cancelled | Return to input state |

**UX Implementation:**
- Poll this every 500ms during active transfer
- Show dual progress:
  - File progress: `current_file` + percentage
  - Overall progress: `files_completed / total_files`
- Show transfer speed (calculate from bytes/time)

---

#### `p2p_is_receiving`
**Purpose:** Quick check if currently downloading.

**Returns:** `boolean`

---

### Utility Commands

#### `p2p_create_mod_pack_preview`
**Purpose:** Generate pack info without starting share (for preview).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | ✅ | Pack name |
| `description` | string | ✅ | Pack description |
| `mod_paths` | string[] | ✅ | Mod file paths |
| `creator` | string | ❌ | Creator name |

**Returns:** `ShareableModPack` object:
```json
{
  "name": "My Iron Man Pack",
  "description": "Custom Iron Man skins",
  "mods": [
    {
      "filename": "IronMan_Skin_9999999_P.pak",
      "size": 52428800,
      "hash": "sha256...",
      "iostore_files": [
        { "extension": "ucas", "size": 51200000, "hash": "..." },
        { "extension": "utoc", "size": 1228800, "hash": "..." }
      ]
    }
  ],
  "created_at": 1701398400,
  "creator": "ModderName"
}
```

**UX Implementation:**
- Show before confirming share
- Display file list with sizes
- Show total pack size
- Warn if pack is very large

---

#### `p2p_validate_connection_string`
**Purpose:** Check if a connection string is valid format.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `connection_string` | string | ✅ | String to validate |

**Returns:** `boolean` (or throws error with reason)

**UX Implementation:**
- Validate on input blur or as user types
- Show ✓ if valid, ✗ with error if invalid
- Enable/disable Connect button based on validity

---

#### `p2p_hash_file`
**Purpose:** Calculate SHA256 hash of a file.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `file_path` | string | ✅ | Full path to file |

**Returns:** `string` (hex-encoded SHA256)

**UX Implementation:**
- Use for verification UI if needed
- Can show "File verified ✓" after download

---

## UI Component Recommendations

### Share Panel (Host)

```
┌──────────────────────────────────────────────────┐
│  📤 Share Mods                            [×]    │
├──────────────────────────────────────────────────┤
│                                                  │
│  Pack Name: [_________________________]          │
│                                                  │
│  Description:                                    │
│  [_________________________________________]     │
│  [_________________________________________]     │
│                                                  │
│  Selected Mods (3):                              │
│  ┌────────────────────────────────────────┐      │
│  │ ☑ IronMan_Skin.pak         (52 MB)    │      │
│  │ ☑ SpiderMan_Skin.pak       (48 MB)    │      │
│  │ ☑ Hulk_Skin.pak            (61 MB)    │      │
│  └────────────────────────────────────────┘      │
│                                                  │
│  Total Size: 161 MB                              │
│                                                  │
│  Creator (optional): [________________]          │
│                                                  │
│           [Cancel]   [Start Sharing]             │
└──────────────────────────────────────────────────┘
```

### Active Share Display

```
┌──────────────────────────────────────────────────┐
│  🟢 Sharing Active                               │
├──────────────────────────────────────────────────┤
│                                                  │
│  Share Code:                                     │
│  ┌────────────────────────────────────────┐      │
│  │     A3B7-K9M2-X5P8                     │ 📋   │
│  └────────────────────────────────────────┘      │
│                                                  │
│  Pack: My Iron Man Collection                    │
│  Mods: 3 files (161 MB)                          │
│                                                  │
│  Share this code with others to let them         │
│  download your mod pack.                         │
│                                                  │
│              [Stop Sharing]                      │
└──────────────────────────────────────────────────┘
```

### Receive Panel (Client)

```
┌──────────────────────────────────────────────────┐
│  📥 Receive Mods                          [×]    │
├──────────────────────────────────────────────────┤
│                                                  │
│  Enter Share Code:                               │
│  [____-____-____]  or paste full code            │
│                                                  │
│  [_________________________________________] ✓   │
│                                                  │
│           [Cancel]   [Connect]                   │
└──────────────────────────────────────────────────┘
```

### Transfer Progress

```
┌──────────────────────────────────────────────────┐
│  📥 Downloading...                               │
├──────────────────────────────────────────────────┤
│                                                  │
│  Pack: My Iron Man Collection                    │
│  From: ModderName                                │
│                                                  │
│  Current: IronMan_Skin.pak                       │
│  ████████████░░░░░░░░░░░░░░░░░  45%              │
│                                                  │
│  Overall: 2 of 5 files                           │
│  ██████████████░░░░░░░░░░░░░░░  40%              │
│                                                  │
│  Speed: 12.5 MB/s                                │
│  ETA: ~8 seconds                                 │
│                                                  │
│              [Cancel]                            │
└──────────────────────────────────────────────────┘
```

---

## Security Notes for Users

Display these in UI or help tooltips:

- 🔒 **End-to-End Encrypted:** All transfers use AES-256 encryption
- ✅ **Integrity Verified:** Files are verified with SHA256 hashes
- 🔑 **Unique Session Keys:** Each share session uses a new random key
- ⚠️ **Only share with trusted users:** The share code grants access to download

---

## Error Handling

| Error | User Message | Recovery Action |
|-------|--------------|-----------------|
| Connection refused | "Could not connect. Check if host is still sharing." | Retry or contact host |
| Invalid code | "Invalid share code format" | Check code and re-enter |
| Hash mismatch | "File verification failed. The file may be corrupted." | Retry download |
| Timeout | "Connection timed out" | Retry or check network |
| Port in use | "Cannot start sharing. Port unavailable." | Retry (tries different ports) |

---

## Accessibility Considerations

1. **Keyboard Navigation:** All share/receive actions should be keyboard accessible
2. **Screen Readers:** Progress updates should be announced
3. **Color Contrast:** Status indicators need text labels, not just colors
4. **Copy Button:** Should work with keyboard and announce success

---

## Future Enhancements (Backend Ready)

1. **Multiple Peers:** Infrastructure supports multiple simultaneous downloads
2. **Resume Downloads:** Hash verification enables future resume capability
3. **Pack Versioning:** `created_at` timestamp enables version tracking
4. **Creator Profiles:** Creator field can link to user profiles
