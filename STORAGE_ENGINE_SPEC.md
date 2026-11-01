# Storage Invariants & Crash Consistency

### Write-Ahead Log (WAL) Group Commit
- Coordinated microsecond WAL batching with synchronized `fdatasync` disk flushes.
- Guaranteed ACID crash consistency across abnormal process termination.

