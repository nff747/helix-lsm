# Helix-LSM Architecture Specification

Helix-LSM is an industrial-grade Log-Structured Merge-tree storage engine designed for zero-allocation point queries and high-throughput write ingestion.

## Key Architectural Invariants

### 1. Write Path
- **WAL Ingestion**: Mutations are sequentially appended to a 4KiB page-aligned Write-Ahead Log before hitting the concurrent `SkiplistMemtable`.
- **Atomic MemTable Freeze**: When memory capacity exceeds the threshold, the active MemTable is swapped into an immutable SSTable builder.

### 2. Read Path
- **Multi-Level Point Queries**: Lookups query `MemTable` -> `BlockCache` -> `BloomFilter` -> `SparseBlockIndex` -> `DataBlock`.
- **Negative Lookup Elimination**: 10-bit-per-key counting Bloom filters eliminate disk I/O on 99% of non-existent keys.

### 3. Compaction Pipeline
- **K-Way Merge Heap**: Multi-stream compaction uses a Min-Heap `BinaryHeap` iterator for $O(N \log K)$ sorted interleaving.
- **Tombstone GC**: Deletion markers reaching the bottom-most level are physically discarded to reclaim storage.
