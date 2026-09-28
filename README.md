# 🧬 helix-lsm

> Lightweight, high-performance Log-Structured Merge-tree (LSM) embedded storage engine in Rust.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org/)

`helix-lsm` is an embedded key-value storage engine engineered for write-heavy workloads with predictable latency, crash-resilient write-ahead logging, and background SSTable compaction.

---

## 🏗️ Architecture

```
   Writes (Put/Delete)
          │
          ├──► Write-Ahead Log (WAL) ──► fsync (crash recovery)
          │
          └──► MemTable (In-Memory SkipList / BTreeMap)
                     │
               (When size >= threshold)
                     ▼
          Immutable SSTables on Disk (Sorted String Tables)
          ┌──────────────────────────────────────────────┐
          │  Data Block  |  Index Block  |  Bloom Filter │
          └──────────────────────────────────────────────┘
                     │
               (Background)
                     ▼
          Compactor (K-Way Merge & Tombstone Pruning)
```

### Components
- **Write-Ahead Log (`wal.rs`)**: Sequential append-only journal guaranteeing durability and zero-loss crash recovery.
- **MemTable (`memtable.rs`)**: Fast concurrent in-memory write buffer backed by ordered indexing.
- **SSTable (`sstable.rs`)**: Immutable on-disk sorted string tables with block-level binary search and trailing index offsets.
- **Bloom Filters (`bloom.rs`)**: Bit-array probabilistic filter preventing negative-read disk seeks.
- **Compactor (`compactor.rs`)**: Multi-way merge compactor combining tiered SSTables and purging tombstones.

---

## 🚀 Quick Start

### Build and Test

```bash
# Run unit & integration test suite
cargo test

# Run release build
cargo build --release
```

### Usage

```rust
use helix_lsm::LsmEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = LsmEngine::open("./data")?;

    // Writes
    engine.put(b"user:1001", b"{\"name\":\"Alice\"}")?;
    
    // Reads
    if let Some(val) = engine.get(b"user:1001")? {
        println!("Value: {}", String::from_utf8_lossy(&val));
    }

    // Deletes (tombstone records)
    engine.delete(b"user:1001")?;

    Ok(())
}
```

---

## 📄 License

MIT © [nff747](https://github.com/nff747)
