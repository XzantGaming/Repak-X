# P2P Internet-Wide Implementation TODO

## ✅ Completed

### Core Infrastructure
- [x] Added libp2p dependencies to Cargo.toml
- [x] Created `p2p_libp2p.rs` with full NAT traversal support
  - Kademlia DHT for peer discovery
  - DCUtR for hole punching
  - Relay protocol for fallback
  - AutoNAT for NAT detection
  - Noise encryption for transport
- [x] Created `p2p_manager.rs` integration layer
- [x] Added modules to main_tauri.rs
- [x] Comprehensive documentation (P2P_INTERNET_WIDE.md)
- [x] Quick start guide (P2P_QUICKSTART.md)

## 🚧 In Progress - Critical Path

### 1. File Transfer Integration
**Priority: HIGH**

The libp2p network layer is ready, but we need to connect it to the actual file transfer:

```rust
// TODO: In p2p_manager.rs
impl UnifiedP2PManager {
    async fn handle_file_transfer(&mut self, peer_id: PeerId) -> P2PResult<()> {
        // 1. Get libp2p stream to peer
        // 2. Use existing p2p_sharing encryption/transfer logic
        // 3. Send files over libp2p stream instead of raw TCP
        // 4. Update progress tracking
    }
}
```

**Files to modify:**
- `p2p_manager.rs` - Add stream handling
- `p2p_sharing.rs` - Abstract TCP to work with any stream

### 2. Stream Protocol Definition
**Priority: HIGH**

Define how files are transferred over libp2p streams:

```rust
// TODO: Create p2p_protocol.rs
use libp2p::request_response::{ProtocolSupport, RequestResponse};

// Define request/response protocol for file chunks
// Integrate with existing P2PMessage enum
```

### 3. Update Tauri Commands
**Priority: MEDIUM**

Modify existing commands to use UnifiedP2PManager:

```rust
// TODO: In main_tauri.rs

// Replace P2PState with UnifiedP2PState
struct UnifiedP2PState {
    manager: Mutex<p2p_manager::UnifiedP2PManager>,
}

// Update all p2p_* commands to use new manager
#[tauri::command]
async fn p2p_start_sharing(
    name: String,
    description: String,
    mod_paths: Vec<String>,
    creator: Option<String>,
    p2p_state: State<'_, UnifiedP2PState>,
) -> Result<String, String> {
    let paths: Vec<PathBuf> = mod_paths.iter().map(PathBuf::from).collect();
    let mut manager = p2p_state.manager.lock().unwrap();
    
    let share_info = manager.start_sharing(name, description, paths, creator)
        .await
        .map_err(|e| e.to_string())?;
    
    share_info.encode().map_err(|e| e.to_string())
}
```

## 📋 TODO - Features

### Phase 1: Basic Functionality

- [ ] **Stream-based file transfer**
  - Adapt existing TCP transfer to libp2p streams
  - Maintain encryption and integrity checks
  - Test with small files first

- [ ] **Progress tracking**
  - Update TransferProgress with connection type (direct/relay)
  - Add peer discovery progress
  - Show NAT traversal status

- [ ] **Error handling**
  - Graceful fallback to relay
  - Retry logic for failed connections
  - User-friendly error messages

- [ ] **Connection state management**
  - Track active connections
  - Clean up on disconnect
  - Reconnection logic

### Phase 2: Reliability

- [ ] **Resume support**
  - Save transfer state
  - Resume from last chunk
  - Verify partial files

- [ ] **Multiple simultaneous transfers**
  - Download from multiple peers
  - Upload to multiple peers
  - Bandwidth management

- [ ] **Connection quality monitoring**
  - Measure bandwidth
  - Detect slow connections
  - Switch to better peers

### Phase 3: User Experience

- [ ] **UI updates**
  - Show peer ID instead of IP
  - Display connection type (direct/relay)
  - NAT status indicator
  - Connection quality meter

- [ ] **Share code improvements**
  - QR code generation
  - One-click copy
  - Expiration time
  - Password protection (optional)

- [ ] **Relay server management**
  - List of public relays
  - Add custom relays
  - Test relay connectivity
  - Relay performance metrics

### Phase 4: Advanced Features

- [ ] **Multi-source downloads**
  - BitTorrent-style swarming
  - Download chunks from multiple peers
  - Automatic peer selection

- [ ] **Content addressing**
  - IPFS-style content IDs
  - Deduplicate common files
  - Verify by hash

- [ ] **Reputation system**
  - Track reliable peers
  - Prefer fast connections
  - Block bad actors

- [ ] **Bandwidth marketplace**
  - Optional: Earn tokens for relaying
  - Incentivize relay operators
  - Premium relay services

## 🔧 Technical Debt

### Code Quality

- [ ] **Error handling**
  - Replace unwrap() with proper error handling
  - Add context to errors
  - Implement retry logic

- [ ] **Testing**
  - Unit tests for each module
  - Integration tests for full flow
  - NAT traversal simulation tests
  - Stress tests with large files

- [ ] **Documentation**
  - Add inline documentation
  - API documentation
  - Architecture diagrams
  - Sequence diagrams

- [ ] **Performance**
  - Profile memory usage
  - Optimize DHT queries
  - Reduce connection overhead
  - Benchmark transfer speeds

### Security

- [ ] **Security audit**
  - Review encryption implementation
  - Check for timing attacks
  - Validate peer authentication
  - Test against malicious peers

- [ ] **Privacy enhancements**
  - Optional: Tor integration
  - Optional: I2P support
  - Metadata protection
  - Traffic analysis resistance

## 🐛 Known Issues

### Critical

- [ ] File transfer not yet implemented over libp2p streams
- [ ] Progress tracking needs update for async operations
- [ ] Connection string format needs validation

### Important

- [ ] No relay server list yet (peers can relay for each other, but dedicated relays would help)
- [ ] DHT bootstrap needs known peers (currently empty)
- [ ] No connection timeout handling
- [ ] Memory usage not optimized for large files

### Minor

- [ ] Log messages need cleanup
- [ ] Error messages not user-friendly
- [ ] No connection statistics
- [ ] No bandwidth limiting

## 📝 Implementation Notes

### File Transfer Over libp2p

The key challenge is adapting the existing TCP-based file transfer to work with libp2p streams:

```rust
// Current (TCP):
let mut stream = TcpStream::connect(&addr)?;
stream.write_all(&data)?;

// Target (libp2p):
let stream = swarm.new_stream(peer_id, protocol)?;
stream.write_all(&data).await?;
```

**Approach:**
1. Create a trait for stream operations
2. Implement for both TcpStream and libp2p Stream
3. Update p2p_sharing.rs to use trait instead of TcpStream

### DHT Bootstrap

Need to add bootstrap peers for DHT to work:

```rust
// Option 1: Public IPFS bootstrap nodes
"/dnsaddr/bootstrap.libp2p.io/p2p/..."

// Option 2: Run our own bootstrap nodes
"/ip4/bootstrap.repak.example.com/tcp/4001/p2p/..."

// Option 3: Use mDNS for local discovery + DHT for internet
```

### Relay Servers

While peers can relay for each other, dedicated relays improve reliability:

```rust
// Public relay addresses to add:
const PUBLIC_RELAYS: &[&str] = &[
    "/ip4/relay1.repak.example.com/tcp/4001/p2p/...",
    "/ip4/relay2.repak.example.com/tcp/4001/p2p/...",
];
```

## 🎯 Milestones

### Milestone 1: Basic Internet-Wide Sharing
**Target: 1-2 weeks**
- [ ] File transfer over libp2p streams
- [ ] Basic DHT peer discovery
- [ ] Simple relay fallback
- [ ] Update Tauri commands
- [ ] Test with 2 peers on different networks

### Milestone 2: Production Ready
**Target: 3-4 weeks**
- [ ] Comprehensive error handling
- [ ] Progress tracking
- [ ] Resume support
- [ ] UI updates
- [ ] Documentation
- [ ] Testing suite

### Milestone 3: Advanced Features
**Target: 2-3 months**
- [ ] Multi-source downloads
- [ ] Relay server management
- [ ] Connection quality monitoring
- [ ] Performance optimizations
- [ ] Security audit

## 🚀 Quick Wins

These can be done quickly for immediate value:

1. **Add bootstrap peers** (30 minutes)
   - Use public IPFS bootstrap nodes
   - Test DHT connectivity

2. **Basic stream adapter** (2-3 hours)
   - Create trait for stream operations
   - Implement for TcpStream
   - Prepare for libp2p streams

3. **Connection string validation** (1 hour)
   - Detect old vs new format
   - Show helpful error messages
   - Add format migration

4. **Logging improvements** (1 hour)
   - Add structured logging
   - Connection status logs
   - Performance metrics

## 📚 Resources

### libp2p Documentation
- [libp2p Rust Docs](https://docs.rs/libp2p/)
- [libp2p Concepts](https://docs.libp2p.io/concepts/)
- [NAT Traversal](https://docs.libp2p.io/concepts/nat/)

### Example Projects
- [rust-libp2p examples](https://github.com/libp2p/rust-libp2p/tree/master/examples)
- [IPFS implementation](https://github.com/ipfs/rust-ipfs)

### Testing Tools
- [libp2p test-plans](https://github.com/libp2p/test-plans)
- NAT simulation tools

## 🤝 Contributing

Want to help? Pick a task from TODO and:

1. Comment on the issue (or create one)
2. Fork the repo
3. Create a feature branch
4. Implement the feature
5. Add tests
6. Submit PR

**Good first issues:**
- Add bootstrap peers
- Improve error messages
- Add logging
- Write tests
- Update documentation

---

**Status:** Core infrastructure complete, file transfer integration in progress.

**Next step:** Implement file transfer over libp2p streams (see "In Progress" section).
