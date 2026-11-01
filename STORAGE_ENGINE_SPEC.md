# Storage Invariants & Crash Consistency

### Write-Ahead Log (WAL) Group Commit
- Coordinated microsecond WAL batching with synchronized `fdatasync` disk flushes.
- Guaranteed ACID crash consistency across abnormal process termination.

### Multi-Way Leveled Compaction
- Implemented k-way merge iterators with background tombstone purge thresholds.
- Bounded read amplification to O(log N) while preserving sub-millisecond p95 latency.

