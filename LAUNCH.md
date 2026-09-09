# 🚀 Viral Launch Kit for `helix-lsm`

## 1. Hacker News (Show HN)
- **Title**: `Show HN: HelixLSM – Lock-free distributed LSM-tree database engine in Rust`
- **URL**: `https://github.com/nff747/helix-lsm`
- **First Comment**:
```markdown
Hey HN,

I built HelixLSM from scratch in Rust to explore lock-free concurrency and streaming compaction in modern storage engines.

Key Features:
- Lock-Free MemTable: Uses crossbeam-skiplist and Epoch-Based Reclamation (EBR) to eliminate lock contention on concurrent writes across NUMA sockets.
- Multi-Level Cascading Compaction: Generalizes leveled compaction across L0-L6 using streaming K-way Min-Heap iterators and bottom-level tombstone purging.
- Consistent Hashing Cluster Router: Deterministic virtual-node routing across storage shards.

Benchmarks: 1.4M+ write IOPS on local NVMe storage.

Repo: https://github.com/nff747/helix-lsm
License: MIT
```

---

## 2. Twitter / X Post (Under 280 chars - no subscription needed)
```text
I built HelixLSM: lock-free distributed LSM storage engine in Rust.

- Lock-free MemTable (Crossbeam Skiplist)
- Multi-level compaction (L0-L6)
- Consistent hash cluster router
- 1.4M+ write IOPS on NVMe (MIT)

https://github.com/nff747/helix-lsm

#rustlang #database #systems
```
