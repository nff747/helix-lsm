//! Telemetry and IO Amplification metrics for compaction runs.

#[derive(Debug, Default, Clone)]
pub struct CompactionStats {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub num_tables_compacted: usize,
}

impl CompactionStats {
    pub fn record(&mut self, read: u64, written: u64, tables: usize) {
        self.bytes_read += read;
        self.bytes_written += written;
        self.num_tables_compacted += tables;
    }

    pub fn write_amplification(&self) -> f64 {
        if self.bytes_read == 0 {
            return 1.0;
        }
        self.bytes_written as f64 / self.bytes_read as f64
    }
}
