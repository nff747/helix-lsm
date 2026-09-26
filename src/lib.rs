//! # Helix-LSM
//!
//! A high-performance, embedded Log-Structured Merge (LSM) Tree storage engine in pure Rust.
//!
//! ## Architecture
//! - **Write-Ahead Log (WAL)**: Crash-resilient sequential append log with CRC32 checksums.
//! - **MemTable**: Fast in-memory buffer using BTreeMap with byte-level threshold tracking.
//! - **SSTables**: Immutable sorted string tables with sparse block index and Bloom filters.
//! - **Compactor**: Merges multi-generation SSTables and prunes deleted tombstones.

pub mod bloom;
pub mod compactor;
pub mod memtable;
pub mod sstable;
pub mod wal;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use compactor::Compactor;
use memtable::MemTable;
use sstable::{SsTableReader, SsTableWriter};
use wal::{RecordType, WalReader, WalWriter};

#[derive(Debug, Clone)]
pub struct LsmConfig {
    pub memtable_threshold_bytes: usize,
    pub sstable_compaction_threshold: usize,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            memtable_threshold_bytes: 64 * 1024, // 64 KB
            sstable_compaction_threshold: 4,     // compact after 4 SSTables
        }
    }
}

pub struct LsmEngine {
    dir: PathBuf,
    config: LsmConfig,
    memtable: MemTable,
    wal: WalWriter,
    sstables: Vec<SsTableReader>,
    next_sst_id: u64,
}

impl LsmEngine {
    /// Opens an LSM storage engine directory. Replays WAL on startup to recover unpersisted writes.
    pub fn open<P: AsRef<Path>>(dir: P, config: LsmConfig) -> io::Result<Self> {
        let dir_buf = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir_buf)?;

        let wal_path = dir_buf.join("active.wal");
        let mut memtable = MemTable::new();

        // 1. Recover uncommitted writes from WAL
        if wal_path.exists() {
            let recovered = WalReader::recover(&wal_path)?;
            for (rec_type, key, val) in recovered {
                match rec_type {
                    RecordType::Put => {
                        if let Some(v) = val {
                            memtable.put(key, v);
                        }
                    }
                    RecordType::Delete => {
                        memtable.delete(key);
                    }
                }
            }
        }

        let wal = WalWriter::open(&wal_path)?;

        // 2. Discover existing SSTables in directory: 00001.sst, 00002.sst...
        let mut sst_files: Vec<(u64, PathBuf)> = Vec::new();
        for entry in fs::read_dir(&dir_buf)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("sst") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(id) = stem.parse::<u64>() {
                        sst_files.push((id, path));
                    }
                }
            }
        }

        // Sort by ID ascending (older to newer)
        sst_files.sort_by_key(|(id, _)| *id);
        let next_sst_id = sst_files.last().map(|(id, _)| id + 1).unwrap_or(1);

        let mut sstables = Vec::new();
        for (_, path) in sst_files {
            sstables.push(SsTableReader::open(path)?);
        }

        Ok(Self {
            dir: dir_buf,
            config,
            memtable,
            wal,
            sstables,
            next_sst_id,
        })
    }

    /// Appends key-value pair to WAL and updates active MemTable.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        self.wal.append(RecordType::Put, key, Some(value))?;
        self.memtable.put(key.to_vec(), value.to_vec());

        if self.memtable.byte_size() >= self.config.memtable_threshold_bytes {
            self.flush()?;
        }
        Ok(())
    }

    /// Writes tombstone to WAL and marks key as deleted in MemTable.
    pub fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        self.wal.append(RecordType::Delete, key, None)?;
        self.memtable.delete(key.to_vec());

        if self.memtable.byte_size() >= self.config.memtable_threshold_bytes {
            self.flush()?;
        }
        Ok(())
    }

    /// Point-lookup: checks MemTable -> SSTables (newest to oldest).
    pub fn get(&self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        // 1. Check in-memory active MemTable
        if let Some(val_opt) = self.memtable.get(key) {
            return Ok(val_opt.map(|v| v.to_vec()));
        }

        // 2. Check SSTables from newest to oldest
        for sst in self.sstables.iter().rev() {
            if let Some(result) = sst.get(key)? {
                return Ok(result);
            }
        }

        Ok(None)
    }

    /// Flushes in-memory MemTable to a new SSTable file on disk and truncates WAL.
    pub fn flush(&mut self) -> io::Result<()> {
        if self.memtable.is_empty() {
            return Ok(());
        }

        let sst_path = self.dir.join(format!("{:05}.sst", self.next_sst_id));
        self.next_sst_id += 1;

        let entries = std::mem::take(&mut self.memtable).into_sorted_vec();
        SsTableWriter::create(&sst_path, &entries)?;

        // Add newly created reader
        self.sstables.push(SsTableReader::open(&sst_path)?);

        // Reset WAL file
        let wal_path = self.dir.join("active.wal");
        drop(std::mem::replace(&mut self.wal, WalWriter::open(&wal_path)?));
        let _ = fs::remove_file(&wal_path);
        self.wal = WalWriter::open(&wal_path)?;

        // Check if compaction is warranted
        if self.sstables.len() >= self.config.sstable_compaction_threshold {
            self.compact()?;
        }

        Ok(())
    }

    /// Compacts all SSTables into a single merged SSTable.
    pub fn compact(&mut self) -> io::Result<()> {
        if self.sstables.len() <= 1 {
            return Ok(());
        }

        let sst_paths: Vec<PathBuf> = (1..self.next_sst_id)
            .map(|id| self.dir.join(format!("{:05}.sst", id)))
            .filter(|p| p.exists())
            .collect();

        if sst_paths.is_empty() {
            return Ok(());
        }

        let compacted_path = self.dir.join(format!("{:05}.sst", self.next_sst_id));
        self.next_sst_id += 1;

        Compactor::compact(&sst_paths, &compacted_path)?;

        // Reopen new single compacted SSTable
        self.sstables.clear();
        if compacted_path.exists() {
            self.sstables.push(SsTableReader::open(&compacted_path)?);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_basic_put_get_delete() {
        let temp_dir = std::env::temp_dir().join("helix_lsm_basic_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut engine = LsmEngine::open(&temp_dir, LsmConfig::default()).unwrap();
        engine.put(b"alpha", b"val_alpha").unwrap();
        engine.put(b"beta", b"val_beta").unwrap();

        assert_eq!(engine.get(b"alpha").unwrap(), Some(b"val_alpha".to_vec()));
        assert_eq!(engine.get(b"beta").unwrap(), Some(b"val_beta".to_vec()));
        assert_eq!(engine.get(b"gamma").unwrap(), None);

        engine.delete(b"alpha").unwrap();
        assert_eq!(engine.get(b"alpha").unwrap(), None);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_engine_crash_recovery_from_wal() {
        let temp_dir = std::env::temp_dir().join("helix_lsm_recovery_test");
        let _ = fs::remove_dir_all(&temp_dir);

        {
            let mut engine = LsmEngine::open(&temp_dir, LsmConfig::default()).unwrap();
            engine.put(b"persist_key", b"persist_val").unwrap();
            // Engine drops without calling flush()
        }

        // Reopen from same directory
        let engine = LsmEngine::open(&temp_dir, LsmConfig::default()).unwrap();
        assert_eq!(engine.get(b"persist_key").unwrap(), Some(b"persist_val".to_vec()));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_engine_flush_and_sstable_read() {
        let temp_dir = std::env::temp_dir().join("helix_lsm_flush_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let mut engine = LsmEngine::open(&temp_dir, LsmConfig::default()).unwrap();
        for i in 0..100 {
            let key = format!("k_{:04}", i);
            let val = format!("v_{:04}", i);
            engine.put(key.as_bytes(), val.as_bytes()).unwrap();
        }

        // Force flush to disk
        engine.flush().unwrap();
        assert_eq!(engine.sstables.len(), 1);

        // Verify data is readable from SSTable
        for i in 0..100 {
            let key = format!("k_{:04}", i);
            let val = format!("v_{:04}", i);
            assert_eq!(engine.get(key.as_bytes()).unwrap(), Some(val.into_bytes()));
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_engine_compaction() {
        let temp_dir = std::env::temp_dir().join("helix_lsm_compaction_test");
        let _ = fs::remove_dir_all(&temp_dir);

        let config = LsmConfig {
            memtable_threshold_bytes: 100, // tiny threshold to trigger frequent flushes
            sstable_compaction_threshold: 3,
        };

        let mut engine = LsmEngine::open(&temp_dir, config).unwrap();
        for i in 0..20 {
            engine.put(format!("key_{}", i).as_bytes(), b"initial_value").unwrap();
        }
        engine.flush().unwrap();

        // Overwrite keys
        for i in 0..10 {
            engine.put(format!("key_{}", i).as_bytes(), b"updated_value").unwrap();
        }
        engine.flush().unwrap();

        // Compact
        engine.compact().unwrap();

        // Verify updated values persisted
        for i in 0..10 {
            assert_eq!(engine.get(format!("key_{}", i).as_bytes()).unwrap(), Some(b"updated_value".to_vec()));
        }
        for i in 10..20 {
            assert_eq!(engine.get(format!("key_{}", i).as_bytes()).unwrap(), Some(b"initial_value".to_vec()));
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
