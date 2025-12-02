# UX Designer Integration Guide - Internet-Wide P2P Sharing

## 📋 Overview

This guide provides everything your UX designer needs to integrate the **internet-wide P2P file sharing** feature into the frontend. All backend functionality is **100% implemented** and ready to use.

## 🎯 What's Available

### ✅ Backend Complete
- Internet-wide P2P networking (libp2p)
- NAT traversal (automatic hole punching)
- Peer discovery (DHT-based)
- File transfer with encryption
- Multi-layer security (7 layers!)
- Progress tracking
- Error handling

### 🎨 Frontend Needed
- UI components for sharing/receiving
- Progress indicators
- Connection status display
- Error message display

---

## 🔌 Tauri Commands Reference

All commands are available via `invoke()` from the frontend.

### 1. Start Sharing (Host)

**Command:** `p2p_start_sharing`

```typescript
interface StartSharingParams {
    name: string;           // Mod pack name
    description: string;    // Mod pack description
    modPaths: string[];    // Array of file paths
    creator?: string;      // Optional creator name
}

interface ShareSession {
    share_code: string;         // Short code for sharing
    encryption_key: string;     // Encryption key (part of connection string)
    local_ip: string;           // Peer ID (not actual IP in internet mode)
    port: number;               // Port (0 for libp2p)
    connection_string: string;  // Full shareable code (base64)
    active: boolean;            // Is session active
}

// Usage
const session: ShareSession = await invoke('p2p_start_sharing', {
    name: "My Awesome Mods",
    description: "Collection of character skins",
    modPaths: [
        "C:\\path\\to\\mod1.pak",
        "C:\\path\\to\\mod2.pak"
    ],
    creator: "YourUsername"
});

console.log("Share this code:", session.connection_string);
```

**UI Elements Needed:**
- Input field for mod pack name
- Text area for description
- File selector (multiple files)
- Input for creator name (optional)
- "Start Sharing" button
- Display area for connection string
- Copy button for connection string

---

### 2. Stop Sharing

**Command:** `p2p_stop_sharing`

```typescript
// Usage
await invoke('p2p_stop_sharing');
```

**UI Elements Needed:**
- "Stop Sharing" button (visible when sharing)
- Confirmation dialog (optional)

---

### 3. Get Share Session Info

**Command:** `p2p_get_share_session`

```typescript
// Usage
const session: ShareSession | null = await invoke('p2p_get_share_session');

if (session) {
    console.log("Currently sharing:", session.share_code);
}
```

**UI Elements Needed:**
- Status indicator (sharing/not sharing)
- Display current share code
- Show connection string

---

### 4. Check if Sharing

**Command:** `p2p_is_sharing`

```typescript
// Usage
const isSharing: boolean = await invoke('p2p_is_sharing');
```

**UI Elements Needed:**
- Status badge/indicator
- Conditional rendering of share panel

---

### 5. Start Receiving (Client)

**Command:** `p2p_start_receiving`

```typescript
interface StartReceivingParams {
    connectionString: string;  // From host
    clientName?: string;       // Optional your name
}

// Usage
await invoke('p2p_start_receiving', {
    connectionString: "eyJwZWVyX2lkIjoiMTJEM0tvb1dS...",
    clientName: "YourUsername"
});
```

**UI Elements Needed:**
- Large text input for connection string
- Paste button
- Input for your name (optional)
- "Start Download" button
- Validation indicator (valid/invalid code)

---

### 6. Stop Receiving

**Command:** `p2p_stop_receiving`

```typescript
// Usage
await invoke('p2p_stop_receiving');
```

**UI Elements Needed:**
- "Cancel Download" button
- Confirmation dialog

---

### 7. Get Transfer Progress

**Command:** `p2p_get_receive_progress`

```typescript
interface TransferProgress {
    current_file: string;      // Currently transferring file
    files_completed: number;   // Number of files done
    total_files: number;       // Total files to transfer
    bytes_transferred: number; // Bytes transferred
    total_bytes: number;       // Total bytes
    status: TransferStatus;    // Current status
}

type TransferStatus = 
    | "Connecting" 
    | "Handshaking" 
    | "Transferring" 
    | "Verifying" 
    | "Complete" 
    | "Failed" 
    | "Cancelled";

// Usage
const progress: TransferProgress | null = await invoke('p2p_get_receive_progress');

if (progress) {
    const percent = (progress.bytes_transferred / progress.total_bytes) * 100;
    console.log(`Progress: ${percent.toFixed(1)}%`);
}
```

**UI Elements Needed:**
- Progress bar (overall)
- Progress bar (current file)
- File counter (e.g., "2 of 5 files")
- Bytes transferred display
- Status text
- Current file name display
- Speed indicator (optional)
- ETA display (optional)

---

### 8. Check if Receiving

**Command:** `p2p_is_receiving`

```typescript
// Usage
const isReceiving: boolean = await invoke('p2p_is_receiving');
```

**UI Elements Needed:**
- Status indicator
- Conditional rendering of progress panel

---

### 9. Validate Connection String

**Command:** `p2p_validate_connection_string`

```typescript
interface ValidateParams {
    connectionString: string;
}

// Usage
const isValid: boolean = await invoke('p2p_validate_connection_string', {
    connectionString: "eyJwZWVyX2lkIjoiMTJEM0tvb1dS..."
});

if (isValid) {
    console.log("Valid connection string!");
}
```

**UI Elements Needed:**
- Real-time validation indicator
- Error message for invalid codes
- Visual feedback (green checkmark / red X)

---

### 10. Create Mod Pack Preview

**Command:** `p2p_create_mod_pack_preview`

```typescript
interface PreviewParams {
    name: string;
    description: string;
    modPaths: string[];
    creator?: string;
}

interface ModPackPreview {
    name: string;
    description: string;
    total_size: number;        // Total bytes
    file_count: number;        // Number of files
    files: FileInfo[];         // File details
}

interface FileInfo {
    filename: string;
    size: number;
    hash: string;
}

// Usage
const preview: ModPackPreview = await invoke('p2p_create_mod_pack_preview', {
    name: "My Mods",
    description: "Cool mods",
    modPaths: ["C:\\mod1.pak", "C:\\mod2.pak"],
    creator: "Me"
});

console.log(`Total size: ${preview.total_size} bytes`);
console.log(`Files: ${preview.file_count}`);
```

**UI Elements Needed:**
- Preview card before sharing
- File list with sizes
- Total size display
- File count display

---

### 11. Hash File (Utility)

**Command:** `p2p_hash_file`

```typescript
interface HashParams {
    filePath: string;
}

// Usage
const hash: string = await invoke('p2p_hash_file', {
    filePath: "C:\\path\\to\\file.pak"
});

console.log("SHA256:", hash);
```

**UI Elements Needed:**
- File integrity verification display (optional)
- Hash comparison tool (optional)

---

## 🎨 UI/UX Recommendations

### Sharing Flow

```
┌─────────────────────────────────────────────────────────┐
│  Step 1: SELECT MODS                                     │
├─────────────────────────────────────────────────────────┤
│  [Select Files Button]                                   │
│  Selected: mod1.pak (2.5 MB)                            │
│           mod2.pak (1.8 MB)                            │
│  Total: 4.3 MB                                          │
│                                                          │
│  [Next]                                                  │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  Step 2: CONFIGURE PACK                                  │
├─────────────────────────────────────────────────────────┤
│  Pack Name: [My Awesome Mods_______________]            │
│  Description:                                            │
│  [_________________________________________]            │
│  [_________________________________________]            │
│                                                          │
│  Creator (optional): [YourName____________]             │
│                                                          │
│  [Back]  [Start Sharing]                                │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  🟢 SHARING ACTIVE                                       │
├─────────────────────────────────────────────────────────┤
│  Share Code:                                             │
│  ┌─────────────────────────────────────────────────┐   │
│  │ eyJwZWVyX2lkIjoiMTJEM0tvb1dS...                 │   │
│  └─────────────────────────────────────────────────┘   │
│  [Copy Code] [Show QR Code]                             │
│                                                          │
│  Status: Waiting for connections...                     │
│  Connected Peers: 0                                     │
│                                                          │
│  [Stop Sharing]                                          │
└─────────────────────────────────────────────────────────┘
```

### Receiving Flow

```
┌─────────────────────────────────────────────────────────┐
│  DOWNLOAD MODS                                           │
├─────────────────────────────────────────────────────────┤
│  Connection String:                                      │
│  ┌─────────────────────────────────────────────────┐   │
│  │ [Paste connection string here]                   │   │
│  └─────────────────────────────────────────────────┘   │
│  [Paste] [Validate]                                     │
│                                                          │
│  Your Name (optional): [_______________]                │
│                                                          │
│  [Start Download]                                        │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  ⏳ DOWNLOADING...                                       │
├─────────────────────────────────────────────────────────┤
│  Status: Transferring                                    │
│  Current File: mod2.pak                                 │
│                                                          │
│  File Progress:                                          │
│  ████████████████████░░░░░░░░ 75%                      │
│                                                          │
│  Overall Progress:                                       │
│  ████████████░░░░░░░░░░░░░░░░ 45%                      │
│                                                          │
│  Files: 2 of 5 complete                                 │
│  Downloaded: 3.2 MB of 7.1 MB                           │
│  Speed: 1.5 MB/s                                        │
│  ETA: 2 minutes                                         │
│                                                          │
│  [Cancel]                                                │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│  ✅ DOWNLOAD COMPLETE                                    │
├─────────────────────────────────────────────────────────┤
│  Successfully downloaded 5 files (7.1 MB)               │
│                                                          │
│  Files saved to: C:\Downloads\Mods\                     │
│                                                          │
│  ✓ mod1.pak (verified)                                  │
│  ✓ mod2.pak (verified)                                  │
│  ✓ mod3.pak (verified)                                  │
│  ✓ mod4.pak (verified)                                  │
│  ✓ mod5.pak (verified)                                  │
│                                                          │
│  [Open Folder] [Done]                                   │
└─────────────────────────────────────────────────────────┘
```

---

## 🎨 Component Suggestions

### 1. SharePanel Component

```typescript
interface SharePanelProps {
    onShare: (params: StartSharingParams) => Promise<void>;
    onStop: () => Promise<void>;
    session: ShareSession | null;
}

// Features:
// - File selection
// - Pack configuration
// - Connection string display
// - Copy button
// - QR code generation
// - Stop button
```

### 2. ReceivePanel Component

```typescript
interface ReceivePanelProps {
    onStart: (connectionString: string, clientName?: string) => Promise<void>;
    onStop: () => Promise<void>;
    progress: TransferProgress | null;
}

// Features:
// - Connection string input
// - Validation indicator
// - Start button
// - Progress display
// - Cancel button
```

### 3. ProgressBar Component

```typescript
interface ProgressBarProps {
    current: number;
    total: number;
    label?: string;
    showPercentage?: boolean;
}

// Features:
// - Animated progress bar
// - Percentage display
// - Label text
// - Color coding (blue = active, green = complete, red = error)
```

### 4. ConnectionStatus Component

```typescript
interface ConnectionStatusProps {
    status: "idle" | "connecting" | "connected" | "error";
    message?: string;
}

// Features:
// - Status indicator (colored dot)
// - Status text
// - Tooltip with details
```

### 5. FileList Component

```typescript
interface FileListProps {
    files: FileInfo[];
    showHashes?: boolean;
    showSizes?: boolean;
}

// Features:
// - List of files
// - File sizes
// - Verification status
// - Icons for file types
```

---

## 🔄 State Management

### Recommended State Structure

```typescript
interface P2PState {
    // Sharing
    isSharing: boolean;
    shareSession: ShareSession | null;
    
    // Receiving
    isReceiving: boolean;
    receiveProgress: TransferProgress | null;
    
    // UI
    currentView: "idle" | "sharing" | "receiving";
    error: string | null;
}

// Actions
type P2PAction =
    | { type: "START_SHARING"; session: ShareSession }
    | { type: "STOP_SHARING" }
    | { type: "START_RECEIVING" }
    | { type: "UPDATE_PROGRESS"; progress: TransferProgress }
    | { type: "STOP_RECEIVING" }
    | { type: "ERROR"; error: string }
    | { type: "CLEAR_ERROR" };
```

### Polling for Progress

```typescript
// Poll progress every 500ms while receiving
useEffect(() => {
    if (!isReceiving) return;
    
    const interval = setInterval(async () => {
        const progress = await invoke('p2p_get_receive_progress');
        if (progress) {
            setReceiveProgress(progress);
            
            // Check if complete
            if (progress.status === "Complete") {
                setIsReceiving(false);
                clearInterval(interval);
            }
        }
    }, 500);
    
    return () => clearInterval(interval);
}, [isReceiving]);
```

---

## 🎨 Color Scheme Suggestions

### Status Colors

```css
/* Idle / Waiting */
--status-idle: #6B7280;

/* Active / Transferring */
--status-active: #3B82F6;

/* Success / Complete */
--status-success: #10B981;

/* Error / Failed */
--status-error: #EF4444;

/* Warning */
--status-warning: #F59E0B;
```

### Progress Colors

```css
/* Progress bar background */
--progress-bg: #E5E7EB;

/* Progress bar fill */
--progress-fill: #3B82F6;

/* Progress bar complete */
--progress-complete: #10B981;
```

---

## 🚨 Error Handling

### Error Types

```typescript
type P2PError =
    | "NETWORK_ERROR"        // Connection failed
    | "VALIDATION_ERROR"     // Invalid input
    | "FILE_ERROR"          // File access error
    | "PROTOCOL_ERROR"      // Protocol mismatch
    | "ENCRYPTION_ERROR"    // Encryption failed
    | "CANCELLED";          // User cancelled

// Error messages
const ERROR_MESSAGES: Record<P2PError, string> = {
    NETWORK_ERROR: "Could not connect. Check your internet connection.",
    VALIDATION_ERROR: "Invalid connection string. Please check and try again.",
    FILE_ERROR: "Could not access file. Check permissions.",
    PROTOCOL_ERROR: "Incompatible version. Both users need the same version.",
    ENCRYPTION_ERROR: "Security error. Please try again.",
    CANCELLED: "Transfer cancelled by user."
};
```

### Error Display

```typescript
// Show error toast/notification
function showError(error: string) {
    // Display error message
    // Auto-dismiss after 5 seconds
    // Allow manual dismiss
}

// Example
try {
    await invoke('p2p_start_sharing', params);
} catch (error) {
    showError(error as string);
}
```

---

## 📱 Responsive Design

### Desktop Layout

```
┌─────────────────────────────────────────────────────────┐
│  [Share Tab] [Receive Tab]                              │
├──────────────────────┬──────────────────────────────────┤
│                      │                                   │
│  Main Content        │  Status Panel                    │
│  (Share/Receive UI)  │  - Connection status             │
│                      │  - Active transfers              │
│                      │  - Recent activity               │
│                      │                                   │
└──────────────────────┴──────────────────────────────────┘
```

### Mobile Layout

```
┌─────────────────────────────────────┐
│  [Share] [Receive]                  │
├─────────────────────────────────────┤
│                                     │
│  Main Content                       │
│  (Full width)                       │
│                                     │
│                                     │
├─────────────────────────────────────┤
│  Status (Collapsible)               │
└─────────────────────────────────────┘
```

---

## 🔔 Notifications

### When to Notify

1. **Share started** - "Now sharing: [Pack Name]"
2. **Connection received** - "User connected to your share"
3. **Download started** - "Downloading: [Pack Name]"
4. **Download complete** - "Download complete! 5 files received"
5. **Error occurred** - "Error: [Error Message]"
6. **Share stopped** - "Stopped sharing"

### Notification Types

```typescript
type NotificationType = "info" | "success" | "warning" | "error";

interface Notification {
    type: NotificationType;
    title: string;
    message: string;
    duration?: number; // Auto-dismiss after ms
}
```

---

## 🧪 Testing Checklist

### Sharing
- [ ] Can select multiple files
- [ ] Can enter pack name and description
- [ ] Connection string is generated
- [ ] Copy button works
- [ ] Can stop sharing
- [ ] Status updates correctly

### Receiving
- [ ] Can paste connection string
- [ ] Validation works (valid/invalid)
- [ ] Download starts
- [ ] Progress updates in real-time
- [ ] Can cancel download
- [ ] Success message shows
- [ ] Files are saved correctly

### Edge Cases
- [ ] Invalid connection string
- [ ] Network disconnection during transfer
- [ ] Large files (>1GB)
- [ ] Many small files (>100)
- [ ] Special characters in filenames
- [ ] Duplicate file names
- [ ] Insufficient disk space

---

## 📚 Example Integration (React)

```typescript
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

function P2PSharePanel() {
    const [isSharing, setIsSharing] = useState(false);
    const [session, setSession] = useState<ShareSession | null>(null);
    const [selectedFiles, setSelectedFiles] = useState<string[]>([]);

    const handleStartSharing = async () => {
        try {
            const session = await invoke('p2p_start_sharing', {
                name: "My Mod Pack",
                description: "Cool mods",
                modPaths: selectedFiles,
                creator: "Me"
            });
            setSession(session);
            setIsSharing(true);
        } catch (error) {
            console.error("Failed to start sharing:", error);
        }
    };

    const handleStopSharing = async () => {
        try {
            await invoke('p2p_stop_sharing');
            setIsSharing(false);
            setSession(null);
        } catch (error) {
            console.error("Failed to stop sharing:", error);
        }
    };

    const handleCopyCode = () => {
        if (session) {
            navigator.clipboard.writeText(session.connection_string);
            // Show "Copied!" toast
        }
    };

    return (
        <div className="p2p-share-panel">
            {!isSharing ? (
                <div>
                    <h2>Share Mods</h2>
                    {/* File selection UI */}
                    <button onClick={handleStartSharing}>
                        Start Sharing
                    </button>
                </div>
            ) : (
                <div>
                    <h2>🟢 Sharing Active</h2>
                    <div className="connection-string">
                        <code>{session?.connection_string}</code>
                        <button onClick={handleCopyCode}>Copy</button>
                    </div>
                    <button onClick={handleStopSharing}>
                        Stop Sharing
                    </button>
                </div>
            )}
        </div>
    );
}
```

---

## 🎯 Summary

### What You Get
- ✅ All backend functionality complete
- ✅ 11 Tauri commands ready to use
- ✅ Comprehensive error handling
- ✅ Progress tracking
- ✅ Security built-in

### What You Need to Build
- 🎨 UI components
- 🎨 State management
- 🎨 Progress indicators
- 🎨 Error displays
- 🎨 Notifications

### Key Points
1. **Use `invoke()`** to call all backend functions
2. **Poll progress** every 500ms while transferring
3. **Handle errors** gracefully with user-friendly messages
4. **Show status** clearly (idle/connecting/transferring/complete)
5. **Validate input** before calling backend

---

## 📞 Support

If you need clarification on any command or functionality:
1. Check the backend code in `src/p2p_*.rs` files
2. See example usage in this guide
3. Test commands in browser console
4. Check logs for detailed error messages

**All backend is ready - just wire it up! 🚀**
