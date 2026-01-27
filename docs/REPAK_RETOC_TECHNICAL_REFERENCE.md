# Repak & Retoc Technical Reference

This document provides a comprehensive technical analysis of the `repak` (PAK file handling) and `retoc` (IoStore container handling) Rust libraries. This serves as the authoritative reference for implementing equivalent functionality in C#.

---

## Table of Contents

1. [PAK File Format Overview](#pak-file-format-overview)
2. [Repak Library Architecture](#repak-library-architecture)
3. [PAK Writing Flow](#pak-writing-flow)
4. [Data Structures](#data-structures)
5. [Encryption](#encryption)
6. [IoStore Format Overview](#iostore-format-overview)
7. [Retoc Library Architecture](#retoc-library-architecture)
8. [Zen Asset Conversion](#zen-asset-conversion-legacy--iostore)
9. [IoStore Bundle Overview](#iostore-bundle-overview)
10. [Companion PAK Creation](#companion-pak-creation)
11. [Summary: Critical Implementation Details](#summary-critical-implementation-details)
12. [Marvel Rivals Specific](#marvel-rivals-specific)

---

## PAK File Format Overview

### File Structure (V11 - Fnv64BugFix)

A PAK file consists of three main sections:

```
┌─────────────────────────────────────────────────────────────┐
│ DATA SECTION                                                │
│   For each entry:                                           │
│     - Entry Record (FPakEntry) - variable size              │
│     - Entry Data (possibly encrypted/compressed)            │
├─────────────────────────────────────────────────────────────┤
│ INDEX SECTION (encrypted for V11)                           │
│   - Mount Point (length-prefixed string)                    │
│   - Entry Count (u32)                                       │
│   - Path Hash Seed (u64) [V10+]                             │
│   - Path Hash Index metadata [V10+]                         │
│   - Full Directory Index metadata [V10+]                    │
│   - Encoded Entries                                         │
│   - Unused File Count (u32)                                 │
├─────────────────────────────────────────────────────────────┤
│ PATH HASH INDEX (encrypted) [V10+]                          │
├─────────────────────────────────────────────────────────────┤
│ FULL DIRECTORY INDEX (encrypted) [V10+]                     │
├─────────────────────────────────────────────────────────────┤
│ 35-BYTE MAGIC BLOCK                                         │
├─────────────────────────────────────────────────────────────┤
│ FOOTER                                                      │
│   - Encryption GUID (16 bytes) [V7+]                        │
│   - Encrypted flag (1 byte) [V4+]                           │
│   - Magic (4 bytes): 0x5A6F12E1                             │
│   - Version (4 bytes): 11                                   │
│   - Index Offset (8 bytes)                                  │
│   - Index Size (8 bytes)                                    │
│   - Index Hash (20 bytes) - SHA1 of index BEFORE encryption │
│   - Compression Methods (5 × 32 bytes) [V8B+]               │
└─────────────────────────────────────────────────────────────┘
```

### Version History

| Version | VersionMajor | Features |
|---------|--------------|----------|
| V0 | Unknown | Unknown |
| V1 | Initial | Initial specification |
| V2 | NoTimestamps | Timestamps removed |
| V3 | CompressionEncryption | Compression and encryption support |
| V4 | IndexEncryption | Index encryption support |
| V5 | RelativeChunkOffsets | Offsets relative to header |
| V6 | DeleteRecords | Record deletion support |
| V7 | EncryptionKeyGuid | Include key GUID |
| V8A | FNameBasedCompression | Compression names (4 slots) |
| V8B | FNameBasedCompression | Compression names (5 slots) |
| V9 | FrozenIndex | Frozen index byte |
| V10 | PathHashIndex | Path hash index |
| V11 | Fnv64BugFix | FNV64 bug fix |

### Footer Size Calculation

```rust
// From lib.rs Version::size()
fn size(self) -> i64 {
    let mut size = 4 + 4 + 8 + 8 + 20;  // magic + version + offset + size + hash = 44
    if self.version_major() >= VersionMajor::EncryptionKeyGuid {
        size += 16;  // encryption uuid
    }
    if self.version_major() >= VersionMajor::IndexEncryption {
        size += 1;   // encrypted flag
    }
    if self.version_major() == VersionMajor::FrozenIndex {
        size += 1;   // frozen index flag
    }
    if self >= Version::V8A {
        size += 32 * 4;  // 4 compression method names
    }
    if self >= Version::V8B {
        size += 32;      // 5th compression method name
    }
    size
}
```

For V11: 44 + 16 + 1 + 160 = **221 bytes**

---

## Repak Library Architecture

### Module Structure

```
repak/src/
├── lib.rs          # Public API, Version/VersionMajor enums, constants
├── pak.rs          # PakBuilder, PakReader, PakWriter, Pak struct, index writing
├── entry.rs        # Entry struct, read/write methods, encoded entry format
├── data.rs         # PartialEntry, encryption, compression, get_limit()
├── footer.rs       # Footer struct, read/write
├── ext.rs          # Extension traits for reading/writing
├── error.rs        # Error types
└── utils.rs        # AesKey parsing
```

### Key Types

#### `Version` (lib.rs)
```rust
pub enum Version {
    V0, V1, V2, V3, V4, V5, V6, V7, V8A, V8B, V9, V10, V11
}
```

#### `VersionMajor` (lib.rs)
```rust
#[repr(u32)]
pub enum VersionMajor {
    Unknown = 0,
    Initial = 1,
    NoTimestamps = 2,
    CompressionEncryption = 3,
    IndexEncryption = 4,
    RelativeChunkOffsets = 5,
    DeleteRecords = 6,
    EncryptionKeyGuid = 7,
    FNameBasedCompression = 8,
    FrozenIndex = 9,
    PathHashIndex = 10,
    Fnv64BugFix = 11,
}
```

#### `Hash` (pak.rs)
```rust
pub struct Hash(pub(crate) [u8; 20]);  // SHA1 hash
```

#### `Entry` (entry.rs)
```rust
pub struct Entry {
    pub offset: u64,
    pub compressed: u64,
    pub uncompressed: u64,
    pub compression_slot: Option<u32>,
    pub timestamp: Option<u64>,
    pub hash: Option<Hash>,
    pub blocks: Option<Vec<Block>>,
    pub flags: u8,
    pub compression_block_size: u32,
}
```

#### `Block` (entry.rs)
```rust
pub struct Block {
    pub start: u64,
    pub end: u64,
}
```

#### `Footer` (footer.rs)
```rust
pub struct Footer {
    pub encryption_uuid: Option<u128>,
    pub encrypted: bool,
    pub magic: u32,
    pub version: Version,
    pub version_major: VersionMajor,
    pub index_offset: u64,
    pub index_size: u64,
    pub hash: Hash,
    pub frozen: bool,
    pub compression: Vec<Option<Compression>>,
}
```

---

## PAK Writing Flow

### High-Level Flow

```
1. PakBuilder::new()
2. PakBuilder::key(aes_key)           # Set AES key for encryption
3. PakBuilder::writer(stream, version, mount_point, path_hash_seed)
4. For each file:
   a. entry_builder.build_entry(compress, data, path)  # Creates PartialEntry
   b. pak_writer.write_entry(path, partial_entry)      # Writes to data section
5. pak_writer.write_index()                            # Writes index + footer
```

### Detailed Flow: `write_entry()` (pak.rs:305-328)

```rust
pub fn write_entry<D: AsRef<[u8]>>(
    &mut self,
    path: String,
    partial_entry: PartialEntry<D>,
) -> Result<(), Error> {
    let stream_position = self.writer.stream_position()?;

    // 1. Build the Entry from PartialEntry
    let entry = partial_entry.build_entry(
        self.pak.version,
        &mut self.pak.compression,
        stream_position,
    )?;

    // 2. Write entry record to data section
    entry.write(
        &mut self.writer,
        self.pak.version,
        crate::entry::EntryLocation::Data,
    )?;

    // 3. Add entry to index
    self.pak.index.add_entry(path, entry);
    
    // 4. Write the actual data
    partial_entry.write_data(&mut self.writer)?;

    Ok(())
}
```

### Detailed Flow: `build_partial_entry()` (data.rs:193-292)

This is the core function that prepares data for writing:

```rust
pub(crate) fn build_partial_entry<D>(
    allowed_compression: &[Compression],
    data: D,
    key: &super::Key,
    path: &str,
) -> Result<PartialEntry<D>> {
    let mut hasher = Sha1::new();
    let mut encrypted = false;
    
    // Check if encryption is enabled
    if let super::Key::Some(_) = key {
        encrypted = true;
    }

    // Handle compression (if enabled)
    let compression = allowed_compression.first().cloned();
    let uncompressed_size = data.as_ref().len() as u64;
    
    let mut data = match compression {
        Some(compression) if uncompressed_size > 0 => {
            // Compress in 64KB blocks
            compression_block_size = 0x10000;
            // ... compress each block ...
            PartialEntryData::Blocks { data: compressed_data, blocks }
        }
        _ => {
            compression = None;
            compression_block_size = 0;
            hasher.update(data.as_ref());
            PartialEntryData::Slice(data)
        }
    };

    // Handle encryption (CRITICAL!)
    if let super::Key::Some(key) = key {
        // Convert to owned Vec for padding
        match data {
            PartialEntryData::Slice(inner) => {
                data = PartialEntryData::Blocks {
                    data: inner.as_ref().to_vec(),
                    blocks: vec![],
                };
            }
            _ => {}
        }

        match &mut data {
            PartialEntryData::Blocks { data, .. } => {
                // Calculate encryption limit based on path hash
                let limit = get_limit(path);
                let limit = if limit > data.len() {
                    pad_zeros_to_alignment(data, 16);
                    data.len()
                } else {
                    limit
                };
                // ONLY encrypt up to limit!
                encrypt(key, &mut data[..limit]);
            }
            _ => unreachable!()
        }
    }

    Ok(PartialEntry {
        compression,
        compressed_size: data.as_ref().len() as u64,
        uncompressed_size,
        compression_block_size,
        data,
        hash: Hash(hasher.finalize().into()),
        encrypted,
    })
}
```

### Critical: `get_limit()` Function (data.rs:10-23)

This function calculates how much of the data to encrypt based on a BLAKE3 hash of the path:

```rust
pub(crate) fn get_limit(path: &str) -> usize {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[0x11, 0x22, 0x33, 0x44]);
    hasher.update(path.to_ascii_lowercase().as_bytes());
    let limit =
        ((u64::from_le_bytes(hasher.finalize().as_bytes()[0..8].try_into().unwrap()) % 0x3d) * 63
            + 319)
            & 0xffffffffffffffc0;
    if limit == 0 {
        0x1000
    } else {
        limit as usize
    }
}
```

**Algorithm:**
1. Create BLAKE3 hasher
2. Update with prefix bytes `[0x11, 0x22, 0x33, 0x44]`
3. Update with lowercase path bytes (ASCII)
4. Take first 8 bytes of hash as u64 (little-endian)
5. Calculate: `((hash % 0x3d) * 63 + 319) & 0xffffffffffffffc0`
6. If result is 0, use 0x1000 (4096)
7. Return as limit

---

## Data Structures

### Entry Record (FPakEntry) - Written to Data Section

For **uncompressed** data in V11 (CompressionEncryption+):

| Offset | Size | Field | Description |
|--------|------|-------|-------------|
| 0 | 8 | offset | 0 for data location, actual offset for index |
| 8 | 8 | compressed | Compressed size (padded for encryption) |
| 16 | 8 | uncompressed | Original uncompressed size |
| 24 | 4 | compression | Compression method (0 = None) |
| 28 | 20 | hash | SHA1 hash of original data |
| 48 | 1 | flags | Bit 0 = encrypted, Bit 1 = deleted |
| 49 | 4 | compression_block_size | Block size (0 for uncompressed) |

**Total: 53 bytes** (for uncompressed data without blocks)

For **compressed** data, add:
- 4 bytes: block count
- 16 bytes per block: start (8) + end (8)

### Entry::write() (entry.rs:156-194)

```rust
pub fn write<W: io::Write>(
    &self,
    writer: &mut W,
    version: super::Version,
    location: EntryLocation,
) -> Result<(), super::Error> {
    // Offset: 0 for Data location, actual offset for Index
    writer.write_u64::<LE>(match location {
        EntryLocation::Data => 0,
        EntryLocation::Index => self.offset,
    })?;
    
    writer.write_u64::<LE>(self.compressed)?;
    writer.write_u64::<LE>(self.uncompressed)?;
    
    // Compression method (0 = None, 1+ = index into compression slots)
    let compression = self.compression_slot.map_or(0, |n| n + 1);
    match compression_index_size(version) {
        CompressionIndexSize::U8 => writer.write_u8(compression as u8)?,
        CompressionIndexSize::U32 => writer.write_u32::<LE>(compression)?,
    }

    // Timestamp (V1 only)
    if version.version_major() == VersionMajor::Initial {
        writer.write_u64::<LE>(self.timestamp.unwrap_or_default())?;
    }
    
    // Hash (always present)
    writer.write_all(&self.hash.unwrap().0)?;
    
    // Blocks, flags, block_size (V3+)
    if version.version_major() >= VersionMajor::CompressionEncryption {
        // Blocks only written if Some (i.e., compression is used)
        if let Some(blocks) = &self.blocks {
            writer.write_u32::<LE>(blocks.len() as u32)?;
            for block in blocks {
                block.write(writer)?;
            }
        }
        writer.write_u8(self.flags)?;
        writer.write_u32::<LE>(self.compression_block_size)?;
    }

    Ok(())
}
```

### Encoded Entry Format (V10+)

Used in the index for compact storage:

```rust
pub fn write_encoded<W: io::Write>(&self, writer: &mut W) -> Result<(), super::Error> {
    // Build flags u32:
    // Bits 0-5: compression_block_size >> 11 (or 0x3f if doesn't fit)
    // Bits 6-21: compression_blocks_count
    // Bit 22: is_encrypted
    // Bits 23-28: compression_slot + 1 (0 = none)
    // Bit 29: is_size_32_bit_safe
    // Bit 30: is_uncompressed_size_32_bit_safe
    // Bit 31: is_offset_32_bit_safe
    
    let flags = (compression_block_size)
        | (compression_blocks_count << 6)
        | ((self.is_encrypted() as u32) << 22)
        | (self.compression_slot.map_or(0, |n| n + 1) << 23)
        | ((is_size_32_bit_safe as u32) << 29)
        | ((is_uncompressed_size_32_bit_safe as u32) << 30)
        | ((is_offset_32_bit_safe as u32) << 31);

    writer.write_u32::<LE>(flags)?;

    // Optional: full compression_block_size if doesn't fit in 6 bits
    if compression_block_size == 0x3f {
        writer.write_u32::<LE>(self.compression_block_size)?;
    }

    // Offset (32 or 64 bit based on flag)
    if is_offset_32_bit_safe {
        writer.write_u32::<LE>(self.offset as u32)?;
    } else {
        writer.write_u64::<LE>(self.offset)?;
    }

    // Uncompressed size (32 or 64 bit based on flag)
    if is_uncompressed_size_32_bit_safe {
        writer.write_u32::<LE>(self.uncompressed as u32)?;
    } else {
        writer.write_u64::<LE>(self.uncompressed)?;
    }

    // Compressed size only if compression is used
    if self.compression_slot.is_some() {
        if is_size_32_bit_safe {
            writer.write_u32::<LE>(self.compressed as u32)?;
        } else {
            writer.write_u64::<LE>(self.compressed)?;
        }

        // Block sizes (only if multiple blocks or encrypted)
        let blocks = self.blocks.as_ref().unwrap();
        if blocks.len() > 1 || self.is_encrypted() {
            for b in blocks {
                writer.write_u32::<LE>((b.end - b.start) as u32)?;
            }
        }
    }

    Ok(())
}
```

---

## Encryption

### AES Key Handling (utils.rs)

```rust
impl std::str::FromStr for AesKey {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let try_parse = |mut bytes: Vec<_>| {
            // CRITICAL: Reverse each 4-byte chunk!
            bytes.chunks_mut(4).for_each(|c| c.reverse());
            aes::Aes256::new_from_slice(&bytes).ok().map(AesKey)
        };
        // Try hex first, then base64
        hex::decode(s.strip_prefix("0x").unwrap_or(s))
            .ok()
            .and_then(try_parse)
            .or_else(|| base64::decode(s).ok().and_then(try_parse))
            .ok_or(crate::Error::Aes)
    }
}
```

**CRITICAL**: The AES key bytes are reversed in 4-byte chunks before creating the cipher!

### Encryption Function (data.rs:33-41)

```rust
pub(crate) fn encrypt(key: &aes::Aes256, bytes: &mut [u8]) {
    use aes::cipher::BlockEncrypt;
    for chunk in bytes.chunks_mut(16) {
        // 1. Reverse each 4-byte chunk BEFORE encryption
        chunk.chunks_mut(4).for_each(|c| c.reverse());
        // 2. Encrypt the block
        key.encrypt_block(aes::Block::from_mut_slice(chunk));
        // 3. Reverse each 4-byte chunk AFTER encryption
        chunk.chunks_mut(4).for_each(|c| c.reverse());
    }
}
```

**CRITICAL**: UE4's AES encryption reverses 4-byte chunks before AND after encryption!

### Decryption Function (data.rs:43-56)

```rust
pub(crate) fn decrypt(key: &super::Key, bytes: &mut [u8]) -> Result<(), super::Error> {
    if let super::Key::Some(key) = key {
        use aes::cipher::BlockDecrypt;
        for chunk in bytes.chunks_mut(16) {
            chunk.chunks_mut(4).for_each(|c| c.reverse());
            key.decrypt_block(aes::Block::from_mut_slice(chunk));
            chunk.chunks_mut(4).for_each(|c| c.reverse());
        }
        Ok(())
    } else {
        Err(super::Error::Encrypted)
    }
}
```

---

## Index Writing (pak.rs:537-717)

### V10+ Index Structure

```rust
fn write<W: Write + Seek>(&self, writer: &mut W, key: &super::Key) -> Result<(), super::Error> {
    let index_offset = writer.stream_position()?;

    let mut index_buf = vec![];
    let mut index_writer = io::Cursor::new(&mut index_buf);
    
    // 1. Mount point
    index_writer.write_string(&self.mount_point)?;

    // 2. Record count
    index_writer.write_u32::<LE>(self.index.entries.len() as u32)?;
    
    // 3. Path hash seed
    index_writer.write_u64::<LE>(path_hash_seed)?;

    // 4. Build encoded entries
    let (encoded_entries, offsets) = {
        let mut offsets = Vec::new();
        let mut encoded_entries = io::Cursor::new(vec![]);
        for entry in self.index.entries.values() {
            offsets.push(encoded_entries.get_ref().len() as u32);
            entry.write_encoded(&mut encoded_entries)?;
        }
        (encoded_entries.into_inner(), offsets)
    };

    // 5. Calculate bytes before path hash index
    let bytes_before_phi = {
        let mut size = 0;
        size += 4;  // mount point length
        size += self.mount_point.len() as u64 + 1;  // mount point + NUL
        size += 4;  // record count
        size += 8;  // path hash seed
        size += 4;  // has path hash index
        size += 8 + 8 + 20;  // phi offset, size, hash
        size += 4;  // has full directory index
        size += 8 + 8 + 20;  // fdi offset, size, hash
        size += 4;  // encoded entries size
        size += encoded_entries.len() as u64;
        size += 4;  // unused file count
        // Pad to 16 for encryption
        if encrypted { size = pad_length(size, 16); }
        size
    };

    let path_hash_index_offset = index_offset + bytes_before_phi;

    // 6. Build and encrypt path hash index
    let mut phi_buf = generate_path_hash_index(...);
    let phi_hash = hash(&phi_buf);  // Hash BEFORE encryption!
    if encrypted {
        pad_zeros_to_alignment(&mut phi_buf, 16);
        encrypt(key, &mut phi_buf);
    }

    let full_directory_index_offset = path_hash_index_offset + phi_buf.len();

    // 7. Build and encrypt full directory index
    let mut fdi_buf = generate_full_directory_index(...);
    let fdi_hash = hash(&fdi_buf);  // Hash BEFORE encryption!
    if encrypted {
        pad_zeros_to_alignment(&mut fdi_buf, 16);
        encrypt(key, &mut fdi_buf);
    }

    // 8. Write index metadata
    index_writer.write_u32::<LE>(1)?;  // has path hash index
    index_writer.write_u64::<LE>(path_hash_index_offset)?;
    index_writer.write_u64::<LE>(phi_buf.len() as u64)?;
    index_writer.write_all(&phi_hash.0)?;

    index_writer.write_u32::<LE>(1)?;  // has full directory index
    index_writer.write_u64::<LE>(full_directory_index_offset)?;
    index_writer.write_u64::<LE>(fdi_buf.len() as u64)?;
    index_writer.write_all(&fdi_hash.0)?;

    // 9. Write encoded entries
    index_writer.write_u32::<LE>(encoded_entries.len() as u32)?;
    index_writer.write_all(&encoded_entries)?;

    // 10. Write unused file count
    index_writer.write_u32::<LE>(0)?;

    // 11. Encrypt index and compute hash
    let mut footer = Footer { ... };
    if encrypted {
        pad_zeros_to_alignment(&mut index_buf, 16);
        footer.hash = hash(&index_buf);  // Hash BEFORE encryption!
        encrypt(key, &mut index_buf);
        footer.encrypted = true;
    } else {
        footer.hash = hash(&index_buf);
    }
    footer.index_size = index_buf.len() as u64;

    // 12. Write everything
    writer.write_all(&index_buf)?;
    writer.write_all(&phi_buf)?;
    writer.write_all(&fdi_buf)?;

    // 13. Write 35-byte magic block
    writer.write_all(&[
        0x06, 0x12, 0x24, 0x20, 0x06, 0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x10, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ])?;

    // 14. Write footer
    footer.write(writer)?;

    Ok(())
}
```

### Path Hash Index Generation (pak.rs:727-743)

```rust
fn generate_path_hash_index<W: Write>(
    writer: &mut W,
    path_hash_seed: u64,
    entries: &BTreeMap<String, Entry>,
    offsets: &Vec<u32>,
) -> Result<(), Error> {
    writer.write_u32::<LE>(entries.len() as u32)?;
    for (path, offset) in entries.keys().zip(offsets) {
        let path_hash = fnv64_path(path, path_hash_seed);
        writer.write_u64::<LE>(path_hash)?;
        writer.write_u32::<LE>(*offset)?;
    }
    writer.write_u32::<LE>(0)?;  // Terminator
    Ok(())
}
```

### FNV64 Hash Functions (pak.rs:745-763)

```rust
fn fnv64<I>(data: I, offset: u64) -> u64
where
    I: IntoIterator<Item = u8>,
{
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut hash = OFFSET.wrapping_add(offset);
    for b in data.into_iter() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn fnv64_path(path: &str, offset: u64) -> u64 {
    let lower = path.to_lowercase();
    // CRITICAL: Use UTF-16 LE encoding!
    let data = lower.encode_utf16().flat_map(u16::to_le_bytes);
    fnv64(data, offset)
}
```

### Full Directory Index Generation (pak.rs:778-807)

```rust
fn generate_full_directory_index<W: Write>(
    writer: &mut W,
    entries: &BTreeMap<String, Entry>,
    offsets: &Vec<u32>,
) -> Result<(), Error> {
    // Build directory structure
    let mut fdi: BTreeMap<&str, BTreeMap<&str, u32>> = Default::default();
    
    for (path, offset) in entries.keys().zip(offsets) {
        // Add all parent directories
        let mut p = path.as_str();
        while let Some((parent, _)) = split_path_child(p) {
            p = parent;
            fdi.entry(p).or_default();
        }

        // Add file to its directory
        let (directory, filename) = split_path_child(path).expect("non-root path");
        fdi.entry(directory).or_default().insert(filename, *offset);
    }

    // Write
    writer.write_u32::<LE>(fdi.len() as u32)?;
    for (directory, files) in &fdi {
        writer.write_string(directory)?;
        writer.write_u32::<LE>(files.len() as u32)?;
        for (filename, offset) in files {
            writer.write_string(filename)?;
            writer.write_u32::<LE>(*offset)?;
        }
    }

    Ok(())
}
```

### Footer Writing (footer.rs:86-117)

```rust
pub fn write<W: std::io::Write>(&self, writer: &mut W) -> Result<(), super::Error> {
    // Encryption GUID (V7+)
    if self.version_major >= VersionMajor::EncryptionKeyGuid {
        writer.write_u128::<LE>(0)?;
    }
    // Encrypted flag (V4+)
    if self.version_major >= VersionMajor::IndexEncryption {
        writer.write_bool(self.encrypted)?;
    }
    // Magic
    writer.write_u32::<LE>(self.magic)?;
    // Version
    writer.write_u32::<LE>(self.version_major as u32)?;
    // Index offset
    writer.write_u64::<LE>(self.index_offset)?;
    // Index size
    writer.write_u64::<LE>(self.index_size)?;
    // Index hash
    writer.write_all(&self.hash.0)?;
    // Frozen flag (V9 only)
    if self.version_major == VersionMajor::FrozenIndex {
        writer.write_bool(self.frozen)?;
    }
    // Compression methods (V8A+: 4, V8B+: 5)
    let algo_size = match self.version {
        ver if ver < Version::V8A => 0,
        ver if ver < Version::V8B => 4,
        _ => 5,
    };
    for i in 0..algo_size {
        let mut name = [0u8; 32];
        if let Some(algo) = self.compression.get(i).cloned().flatten() {
            for (i, b) in algo.to_string().as_bytes().iter().enumerate() {
                name[i] = *b;
            }
        }
        writer.write_all(&name)?;
    }
    Ok(())
}
```

---

## IoStore Format Overview

IoStore is UE5's modern asset container format consisting of:
- `.utoc` - Table of Contents (metadata, chunk info, directory index)
- `.ucas` - Container Archive Store (actual chunk data)
- `.pak` - Companion PAK file (for mod loading, contains chunknames)

### IoStore File Structure

#### .utoc (Table of Contents)

```
┌─────────────────────────────────────────────────────────────┐
│ FIoStoreTocHeader                                           │
│   - Magic, Version, Container ID                            │
│   - Entry counts, compression info                          │
├─────────────────────────────────────────────────────────────┤
│ Chunk IDs (FIoChunkId[])                                    │
│   - 12 bytes each                                           │
├─────────────────────────────────────────────────────────────┤
│ Chunk Offset/Lengths (FIoOffsetAndLength[])                 │
│   - 10 bytes each (5 bytes offset, 5 bytes length)          │
├─────────────────────────────────────────────────────────────┤
│ Perfect Hash Seeds (optional)                               │
├─────────────────────────────────────────────────────────────┤
│ Compression Blocks (FIoStoreTocCompressedBlockEntry[])      │
│   - 12 bytes each                                           │
├─────────────────────────────────────────────────────────────┤
│ Compression Methods (names)                                 │
├─────────────────────────────────────────────────────────────┤
│ Chunk Metas (FIoStoreTocEntryMeta[])                        │
│   - Hash + flags                                            │
├─────────────────────────────────────────────────────────────┤
│ Directory Index                                             │
│   - Mount point, file paths                                 │
└─────────────────────────────────────────────────────────────┘
```

#### .ucas (Container Archive Store)

Raw chunk data, potentially compressed with Oodle. Each chunk is stored as compression blocks.

### Key Structures

#### `FIoChunkId` (12 bytes)
```rust
struct FIoChunkId {
    id: u64,        // Package ID or chunk-specific ID
    index: u16,     // Chunk index within package
    padding: u8,
    chunk_type: u8, // EIoChunkType
}
```

#### `EIoChunkType`
```rust
enum EIoChunkType {
    Invalid = 0,
    ExportBundleData = 1,    // Main package data (.uasset/.uexp combined)
    BulkData = 2,            // .ubulk
    OptionalBulkData = 3,    // .uptnl
    MemoryMappedBulkData = 4,// .m.ubulk
    ScriptObjects = 5,       // Global script objects
    ContainerHeader = 6,     // Package metadata
    ExternalFile = 7,
    ShaderCodeLibrary = 8,
    ShaderCode = 9,
    PackageStoreEntry = 10,
    DerivedData = 11,
    EditorDerivedData = 12,
}
```

#### `FIoOffsetAndLength` (10 bytes)
```rust
// Packed format: 5 bytes offset, 5 bytes length
fn new(offset: u64, length: u64) -> Self {
    // offset stored in first 5 bytes (40 bits)
    // length stored in next 5 bytes (40 bits)
}
```

#### `FIoStoreTocCompressedBlockEntry` (12 bytes)
```rust
struct FIoStoreTocCompressedBlockEntry {
    offset: u40,              // 5 bytes
    compressed_size: u24,     // 3 bytes
    uncompressed_size: u24,   // 3 bytes
    compression_method: u8,   // 1 byte (0=None, 1=Oodle, etc.)
}
```

#### `FPackageId`
```rust
struct FPackageId(u64);

impl FPackageId {
    fn from_name(name: &str) -> Self {
        // CityHash64 of lowercase package name
        FPackageId(cityhasher::hash(name.to_lowercase().as_bytes()))
    }
}
```

### Container Header

The container header chunk contains package metadata for all packages in the container:

```rust
struct FIoContainerHeader {
    container_id: FIoContainerId,
    package_count: u32,
    package_ids: Vec<FPackageId>,
    store_entries: Vec<StoreEntry>,
    // Culture/localization mappings
    // Package redirects
}

struct StoreEntry {
    export_count: i32,
    export_bundle_count: i32,
    load_order: i32,
    pad: i32,
    imported_packages: Vec<FPackageId>,
    shader_map_hashes: Vec<FSHAHash>,
}
```

---

## Retoc Library Architecture

### Module Structure

```
retoc-rivals/src/
├── lib.rs              # Public API, main actions
├── iostore.rs          # IoStore reading
├── iostore_writer.rs   # IoStore writing
├── container_header.rs # FIoContainerHeader
├── compression.rs      # Oodle compression
├── zen.rs              # Zen package format
├── zen_asset_conversion.rs  # Legacy to Zen conversion
├── asset_conversion.rs # Zen to Legacy conversion
├── legacy_asset.rs     # Legacy .uasset/.uexp parsing
├── name_map.rs         # FNameMap handling
├── script_objects.rs   # Script object references
└── ser.rs              # Serialization traits
```

### IoStoreWriter (iostore_writer.rs)

```rust
pub(crate) struct IoStoreWriter {
    toc_path: PathBuf,
    toc_stream: BufWriter<fs::File>,
    cas_stream: BufWriter<fs::File>,
    toc: Toc,
    container_header: Option<FIoContainerHeader>,
    compress_enabled: bool,
}

impl IoStoreWriter {
    pub fn new(toc_path, toc_version, container_header_version, mount_point) -> Self;
    pub fn write_chunk(chunk_id, path, data) -> Result<()>;
    pub fn write_package_chunk(chunk_id, path, data, store_entry) -> Result<()>;
    pub fn finalize() -> Result<()>;
}
```

### IoStoreWriter::write_chunk() Flow

```rust
pub fn write_chunk(&mut self, chunk_id, path, data) -> Result<()> {
    // 1. Add to directory index if path provided
    if let Some(path) = path {
        self.toc.directory_index.add_file(relative_path, chunk_index);
    }

    // 2. Get current offset in .ucas
    let mut offset = self.cas_stream.stream_position()?;
    let start_block = self.toc.compression_blocks.len();

    // 3. Create BLAKE3 hasher for chunk hash
    let mut hasher = blake3::Hasher::new();

    // 4. Write data in compression blocks
    for block in data.chunks(self.toc.compression_block_size) {
        hasher.update(block);  // Hash UNCOMPRESSED data

        // Try Oodle compression if enabled
        let (bytes_to_write, compression_method) = if try_compress {
            // Compress with Oodle, use if smaller
            (compressed, 1u8)
        } else {
            (block, 0u8)
        };

        self.cas_stream.write_all(bytes_to_write)?;

        // Add compression block entry
        self.toc.compression_blocks.push(FIoStoreTocCompressedBlockEntry::new(
            offset, compressed_size, uncompressed_size, compression_method
        ));
        offset += compressed_size;
    }

    // 5. Create chunk meta with BLAKE3 hash
    let hash = hasher.finalize();
    let meta = FIoStoreTocEntryMeta {
        chunk_hash: FIoChunkHash::from_blake3(hash.as_bytes()),
        flags: FIoStoreTocEntryMetaFlags::empty(),
    };

    // 6. Add to TOC
    self.toc.chunks.push(chunk_id);
    self.toc.chunk_offset_lengths.push(FIoOffsetAndLength::new(
        start_block * compression_block_size,
        data.len()
    ));
    self.toc.chunk_metas.push(meta);

    Ok(())
}
```

### IoStoreWriter::finalize() Flow

```rust
pub fn finalize(mut self) -> Result<()> {
    // 1. Write container header chunk (if present)
    if let Some(container_header) = &self.container_header {
        let mut chunk_buffer = vec![];
        container_header.ser(&mut chunk_buffer)?;
        // Align to 16 bytes for AES
        chunk_buffer.resize(align_usize(chunk_buffer.len(), 16), 0);

        let chunk_id = FIoChunkId::create(
            container_header.container_id.0,
            0,
            EIoChunkType::ContainerHeader,
        );
        self.write_chunk(chunk_id, None, &chunk_buffer)?;
    }

    // 2. Serialize TOC to .utoc file
    self.toc_stream.ser(&self.toc)?;

    Ok(())
}
```

---

## Zen Asset Conversion (Legacy → IoStore)

### Conversion Flow

```
1. Read legacy .uasset/.uexp files
2. Parse FLegacyPackageHeader from .uasset
3. Create ZenPackageBuilder
4. setup_zen_package_summary() - Copy package metadata
5. build_zen_import_map() - Convert imports to FPackageObjectIndex
6. build_zen_export_map() - Convert exports to FExportMapEntry
7. build_zen_preload_dependencies() - Build dependency bundles
8. serialize_zen_asset() - Write FZenPackageHeader + export data
9. Write to IoStore container
```

### Key Functions

#### `build_zen_import_map()`
Converts legacy imports to Zen format:
- Script imports → `FPackageObjectIndex::create_script_import()`
- Package imports → `FPackageObjectIndex::create_null()` (not preserved)
- Export imports → `FPackageObjectIndex::create_package_import()`

#### `build_zen_export_map()`
Converts legacy exports to Zen format:
- Calculates `cooked_serial_offset` (relative to header)
- Calculates `public_export_hash` for public exports
- Maps outer/class/super/template indices

#### `build_zen_preload_dependencies()`
Builds export bundles and dependency arcs:
- Sorts exports by dependency order
- Creates export bundle headers and entries
- Builds internal and external dependency arcs

### FZenPackageHeader Structure

```rust
struct FZenPackageHeader {
    summary: FZenPackageSummary,
    name_map: FNameMap,
    bulk_data: Vec<FBulkDataMapEntry>,
    imported_packages: Vec<FPackageId>,
    imported_package_names: Vec<String>,
    imported_public_export_hashes: Vec<u64>,
    import_map: Vec<FPackageObjectIndex>,
    export_map: Vec<FExportMapEntry>,
    export_bundle_headers: Vec<FExportBundleHeader>,
    export_bundle_entries: Vec<FExportBundleEntry>,
    dependency_bundle_headers: Vec<FDependencyBundleHeader>,
    dependency_bundle_entries: Vec<FDependencyBundleEntry>,
    internal_dependency_arcs: Vec<FInternalDependencyArc>,
    external_package_dependencies: Vec<ExternalPackageDependency>,
}
```

---

## IoStore Bundle Overview

An IoStore bundle for Marvel Rivals modding consists of **three files** that must be created together:

```
ModName_P.utoc   - Table of Contents (chunk metadata, directory index)
ModName_P.ucas   - Container Archive Store (actual chunk data, Oodle compressed)
ModName_P.pak    - Companion PAK (chunknames for mod loader recognition)
```

### Why Three Files?

1. **`.utoc`** - Contains metadata about all chunks in the container:
   - Chunk IDs and their offsets/lengths
   - Compression block information
   - Directory index (file paths)
   - Container header with package store entries

2. **`.ucas`** - Contains the actual data:
   - Compressed chunks (Oodle compression)
   - Each package's export bundle data
   - Bulk data (.ubulk content)

3. **`.pak`** - Companion file for mod loader:
   - Required for the game's mod loader to recognize the IoStore bundle
   - Contains a single "chunknames" entry listing all files
   - Must be AES encrypted with the game's key
   - Acts as a "mount aid" for the IoStore container

### File Naming Convention

For mods, the naming convention is:
- `{ModName}_P.utoc` / `{ModName}_P.ucas` / `{ModName}_P.pak`
- The `_P` suffix indicates a patch/mod container
- Priority is determined by numeric suffix (e.g., `_9999999_P` for high priority)

---

## Companion PAK Creation

The companion PAK is a special PAK file that accompanies IoStore bundles. It's created in `iotoc.rs`:

### Purpose

The companion PAK serves as a **mount aid** for the mod loader:
1. The game's mod loader scans for `.pak` files
2. When it finds a PAK, it looks for matching `.utoc`/`.ucas` files
3. The PAK contains a "chunknames" entry listing all files in the IoStore
4. This allows the mod loader to properly mount the IoStore container

### Creation Flow

```rust
// 1. Collect all file paths that were added to the IoStore
let rel_paths = paths.par_iter()
    .map(|p| p.strip_prefix(to_pak_dir).to_slash().to_string())
    .collect::<Vec<_>>();

// 2. Create PAK builder with AES key
let builder = repak::PakBuilder::new()
    .key(AES_KEY.clone().0);

// 3. Create PAK writer with V11, mount point, and path hash seed
let mut pak_writer = builder.writer(
    BufWriter::new(output_file),
    Version::V11,
    pak.mount_point.clone(),           // "../../../"
    Some(pak.path_hash_seed.parse().unwrap()),  // Usually 0
);

// 4. Build the chunknames entry
let entry_builder = pak_writer.entry_builder();
let rel_paths_bytes: Vec<u8> = rel_paths.join("\n").into_bytes();

// 5. Build entry WITHOUT compression (important!)
let entry = entry_builder
    .build_entry(false, rel_paths_bytes, "chunknames")
    .expect("Failed to build entry");

// 6. Write entry and finalize
pak_writer.write_entry("chunknames".to_string(), entry)?;
pak_writer.write_index()?;
```

### Companion PAK Structure

The resulting PAK file has this structure:

```
┌─────────────────────────────────────────────────────────────┐
│ DATA SECTION                                                │
│   Entry Record (53 bytes):                                  │
│     - offset: 0 (u64)                                       │
│     - compressed_size: padded data size (u64)               │
│     - uncompressed_size: original data size (u64)           │
│     - compression: 0 (u32, None)                            │
│     - hash: SHA1 of original data (20 bytes)                │
│     - flags: 1 (u8, encrypted)                              │
│     - block_size: 0 (u32)                                   │
│   Encrypted Data:                                           │
│     - Partially encrypted based on get_limit("chunknames")  │
│     - Padded to 16-byte alignment                           │
├─────────────────────────────────────────────────────────────┤
│ INDEX (encrypted)                                           │
│   - Mount point: "../../../"                                │
│   - Entry count: 1                                          │
│   - Path hash seed: 0                                       │
│   - Path hash index metadata                                │
│   - Full directory index metadata                           │
│   - Encoded entries (1 entry for "chunknames")              │
│   - Unused file count: 0                                    │
├─────────────────────────────────────────────────────────────┤
│ PATH HASH INDEX (encrypted)                                 │
│   - Count: 1                                                │
│   - Entry: (FNV64 hash of "chunknames", offset 0)           │
│   - Terminator: 0                                           │
├─────────────────────────────────────────────────────────────┤
│ FULL DIRECTORY INDEX (encrypted)                            │
│   - Directory count: 1                                      │
│   - Directory "/" with file "chunknames"                    │
├─────────────────────────────────────────────────────────────┤
│ 35-BYTE MAGIC BLOCK                                         │
├─────────────────────────────────────────────────────────────┤
│ FOOTER (221 bytes)                                          │
│   - Encryption GUID: all zeros (16 bytes)                   │
│   - Encrypted flag: 1 (1 byte)                              │
│   - Magic: 0x5A6F12E1 (4 bytes)                             │
│   - Version: 11 (4 bytes)                                   │
│   - Index offset (8 bytes)                                  │
│   - Index size (8 bytes)                                    │
│   - Index hash: SHA1 before encryption (20 bytes)           │
│   - Compression methods: 5 × 32 bytes (all zeros)           │
└─────────────────────────────────────────────────────────────┘
```

### Chunknames Content Format

The "chunknames" entry contains a newline-separated list of all files:

```
Marvel/Content/Meshes/Weapons/SM_WP_1033001_Stick_L.uasset
Marvel/Content/Meshes/Weapons/SM_WP_1033001_Stick_L.uexp
Marvel/Content/Textures/T_MyTexture.uasset
Marvel/Content/Textures/T_MyTexture.uexp
```

### Key Implementation Details

| Aspect | Value |
|--------|-------|
| PAK Version | V11 (Fnv64BugFix) |
| Encryption | AES-256-ECB with UE4 byte swapping |
| Compression | None (uncompressed) |
| Entry Name | "chunknames" |
| Mount Point | "../../../" |
| Path Hash Seed | 0 (or from mod settings) |
| Partial Encryption | Based on `get_limit("chunknames")` |

### get_limit() for "chunknames"

The encryption limit for the "chunknames" path is calculated as:

```rust
fn get_limit(path: &str) -> usize {
    // path = "chunknames"
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[0x11, 0x22, 0x33, 0x44]);
    hasher.update("chunknames".as_bytes());  // Already lowercase
    
    let hash_bytes = hasher.finalize();
    let hash_value = u64::from_le_bytes(hash_bytes[0..8]);
    
    let limit = ((hash_value % 0x3d) * 63 + 319) & 0xffffffffffffffc0;
    if limit == 0 { 0x1000 } else { limit as usize }
}
// Result: Some value between 320 and ~4096 bytes
```

Only the first `limit` bytes of the data are encrypted; the rest remains plaintext.

---

## Summary: Critical Implementation Details

1. **AES Key**: Reverse 4-byte chunks when parsing hex key
2. **AES Encryption**: Reverse 4-byte chunks before AND after encryption
3. **Partial Encryption**: Use `get_limit(path)` with BLAKE3 hash to determine encryption limit
4. **Entry Record**: 53 bytes for uncompressed data (no blocks array)
5. **Hash Timing**: Compute SHA1 hash BEFORE encryption
6. **Index Structure**: Mount point → count → seed → PHI metadata → FDI metadata → encoded entries → 0
7. **35-byte Magic Block**: Fixed bytes before footer
8. **Footer**: 221 bytes for V11 (GUID + encrypted + magic + version + offset + size + hash + 5×32 compression names)
9. **FNV64**: Use UTF-16 LE encoding for path hashing
10. **String Format**: Length-prefixed with NUL terminator

---

## Marvel Rivals Specific

### Game Configuration

| Setting | Value |
|---------|-------|
| **AES Key** | `0C263D8C22DCB085894899C3A3796383E9BF9DE0CBFB08C9BF2DEF2E84F29D74` |
| **Mount Point** | `../../../` |
| **PAK Version** | V11 (Fnv64BugFix) |
| **IoStore TOC Version** | `PerfectHashWithOverflow` (UE5.3) |
| **Container Header Version** | `OptionalSegmentPackages` (UE5.3) |
| **Engine Version** | UE5.3 |
| **Compression** | Oodle (for IoStore), None (for companion PAK) |

### Mod Installation Location

Mods are installed to:
```
{GameInstall}/Marvel/Content/Paks/~mods/
```

### Complete IoStore Bundle Creation Flow

```
1. Collect all .uasset/.uexp files from mod directory
2. For each asset:
   a. Read legacy .uasset header
   b. Read .uexp export data
   c. Read .ubulk bulk data (if exists)
   d. Convert to Zen format (FZenPackageHeader + exports)
   e. Write to IoStore container
3. Write container header chunk
4. Finalize .utoc and .ucas files
5. Create companion .pak file with chunknames
```

### Hash Algorithms Used

| Purpose | Algorithm | Notes |
|---------|-----------|-------|
| Data integrity (PAK) | SHA1 | Entry hashes, index hash |
| Chunk integrity (IoStore) | BLAKE3 | Chunk hashes in TOC |
| Partial encryption limit | BLAKE3 | `get_limit()` function |
| Path hashing (PAK index) | FNV64 | UTF-16 LE encoded paths |
| Package ID | CityHash64 | Lowercase package name |
| Public export hash | CityHash64 | Package-relative export path |

### Typical File Sizes

| File | Typical Size | Notes |
|------|--------------|-------|
| Companion PAK | 500-2000 bytes | Depends on number of files |
| .utoc | 500-5000 bytes | Depends on chunk count |
| .ucas | Varies | Actual asset data, Oodle compressed |

---

## C# Implementation Checklist

When implementing in C#, ensure these are correctly handled:

### PAK Writer (Companion PAK)
- [ ] AES key parsing with 4-byte chunk reversal
- [ ] AES encryption with 4-byte chunk reversal before/after
- [ ] BLAKE3 hashing for `get_limit()` (or use a C# BLAKE3 library)
- [ ] Partial encryption (only encrypt up to limit)
- [ ] Entry record: 53 bytes for uncompressed data
- [ ] SHA1 hash computed on original data before encryption
- [ ] FNV64 path hashing with UTF-16 LE encoding
- [ ] String format: length-prefixed with NUL terminator
- [ ] Index structure with path hash index and full directory index
- [ ] 35-byte magic block before footer
- [ ] Footer: 221 bytes for V11

### IoStore Writer
- [ ] FIoChunkId: 12 bytes (id + index + padding + type)
- [ ] FIoOffsetAndLength: 10 bytes packed format
- [ ] FIoStoreTocCompressedBlockEntry: 12 bytes
- [ ] BLAKE3 hashing for chunk integrity
- [ ] Oodle compression (requires native library)
- [ ] Container header with package store entries
- [ ] Directory index with mount point
- [ ] TOC serialization

### Zen Asset Conversion
- [ ] Legacy package header parsing
- [ ] Name map conversion
- [ ] Import map conversion (script imports, package imports)
- [ ] Export map conversion with public export hashes
- [ ] Dependency bundle generation
- [ ] Export bundle ordering

---

## Reference Files

### Repak Source Files
- `repak/src/lib.rs` - Version enums, MAGIC constant
- `repak/src/pak.rs` - PakBuilder, PakWriter, index writing
- `repak/src/entry.rs` - Entry struct, serialization
- `repak/src/data.rs` - Encryption, compression, get_limit()
- `repak/src/footer.rs` - Footer struct
- `repak/src/ext.rs` - Read/write extensions
- `repak/src/utils.rs` - AES key parsing

### Retoc Source Files
- `retoc-rivals/src/lib.rs` - Main API, action_to_zen()
- `retoc-rivals/src/iostore.rs` - IoStore reading
- `retoc-rivals/src/iostore_writer.rs` - IoStore writing
- `retoc-rivals/src/zen_asset_conversion.rs` - Legacy to Zen conversion
- `retoc-rivals/src/container_header.rs` - FIoContainerHeader
- `retoc-rivals/src/zen.rs` - Zen package structures

### GUI Integration
- `repak-gui/src/install_mod/install_mod_logic/iotoc.rs` - IoStore mod installation
