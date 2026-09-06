use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use bytes::Bytes;

use crate::compaction::CompactionManager;
use crate::memtable::MemTable;
use crate::sstable::{SsTableBuilder, SsTableReader};
use crate::types::{InternalKey, ValueType};
use crate::wal::WalWriter;

pub const DEFAULT_MEMTABLE_THRESHOLD: usize = 4 * 1024 * 1024; // 4MB threshold for flushing

pub struct EngineOptions {
    pub memtable_size_bytes: usize,
    pub sync_wal: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            memtable_size_bytes: DEFAULT_MEMTABLE_THRESHOLD,
            sync_wal: false, // Group commit batching
        }
    }
}

pub struct HelixDb {
    path: PathBuf,
    active_memtable: RwLock<Arc<MemTable>>,
    imm_memtables: RwLock<Vec<Arc<MemTable>>>,
    wal: Mutex<WalWriter>,
    compactor: Mutex<CompactionManager>,
    next_seq: AtomicU64,
    next_mem_id: AtomicU64,
    options: EngineOptions,
}

impl HelixDb {
    pub fn open<P: AsRef<Path>>(path: P, options: EngineOptions) -> io::Result<Arc<Self>> {
        let db_path = path.as_ref().to_path_buf();
        fs::create_dir_all(&db_path)?;

        let wal_path = db_path.join("active.wal");
        let wal = WalWriter::create(&wal_path)?;

        let db = Arc::new(Self {
            path: db_path.clone(),
            active_memtable: RwLock::new(Arc::new(MemTable::new(0))),
            imm_memtables: RwLock::new(Vec::new()),
            wal: Mutex::new(wal),
            compactor: Mutex::new(CompactionManager::new(&db_path)),
            next_seq: AtomicU64::new(1),
            next_mem_id: AtomicU64::new(1),
            options,
        });

        Ok(db)
    }

    /// High-throughput concurrent write:
    /// 1. Assigns monotonic MVCC sequence number.
    /// 2. Appends to Write-Ahead Log (WAL).
    /// 3. Inserts into lock-free MemTable SkipList.
    /// 4. Triggers freeze and flush if memory budget exceeded.
    pub fn put(&self, key: &[u8], value: &[u8]) -> io::Result<u64> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let key_bytes = Bytes::copy_from_slice(key);
        let val_bytes = Bytes::copy_from_slice(value);

        // 1. Append to WAL
        {
            let mut wal = self.wal.lock();
            let internal_key = InternalKey::new(key_bytes.clone(), seq, ValueType::Value);
            wal.append(&internal_key, value)?;
            if self.options.sync_wal {
                wal.sync()?;
            }
        }

        // 2. Insert into lock-free MemTable
        {
            let mem = self.active_memtable.read().clone();
            mem.put(key_bytes, val_bytes, seq);

            // Check if active memtable exceeded flush threshold
            if mem.approximate_size() >= self.options.memtable_size_bytes {
                drop(mem);
                self.rotate_active_memtable()?;
            }
        }

        Ok(seq)
    }

    /// Append deletion tombstone
    pub fn delete(&self, key: &[u8]) -> io::Result<u64> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let key_bytes = Bytes::copy_from_slice(key);

        {
            let mut wal = self.wal.lock();
            let internal_key = InternalKey::new(key_bytes.clone(), seq, ValueType::Deletion);
            wal.append(&internal_key, &[])?;
            if self.options.sync_wal {
                wal.sync()?;
            }
        }

        {
            let mem = self.active_memtable.read().clone();
            mem.delete(key_bytes, seq);
            if mem.approximate_size() >= self.options.memtable_size_bytes {
                drop(mem);
                self.rotate_active_memtable()?;
            }
        }

        Ok(seq)
    }

    /// Read path through LSM hierarchy:
    /// 1. Active lock-free MemTable
    /// 2. Immutable MemTables (in reverse order)
    /// 3. Level 0 SSTables
    /// 4. Level 1+ SSTables
    pub fn get(&self, key: &[u8]) -> io::Result<Option<Bytes>> {
        let snapshot_seq = self.next_seq.load(Ordering::SeqCst);

        // 1. Search Active MemTable (Lock-Free)
        {
            let mem = self.active_memtable.read().clone();
            if let Some(res) = mem.get(key, snapshot_seq) {
                return Ok(res); // Hit value or hit tombstone
            }
        }

        // 2. Search Immutable MemTables
        {
            let imms = self.imm_memtables.read().clone();
            for imm in imms.iter().rev() {
                if let Some(res) = imm.get(key, snapshot_seq) {
                    return Ok(res);
                }
            }
        }

        // 3. Search Level-0 and Level-1 SSTables on disk
        let compactor = self.compactor.lock();
        let manifest = compactor.manifest();

        // Level 0 (overlapping key ranges, search newest first)
        for sst_path in manifest.levels[0].iter().rev() {
            if let Ok(mut reader) = SsTableReader::open(sst_path) {
                if let Some(res) = reader.get(key, snapshot_seq)? {
                    return Ok(res);
                }
            }
        }

        // Level 1 (compacted non-overlapping key ranges)
        for sst_path in manifest.levels[1].iter() {
            if let Ok(mut reader) = SsTableReader::open(sst_path) {
                if let Some(res) = reader.get(key, snapshot_seq)? {
                    return Ok(res);
                }
            }
        }

        Ok(None)
    }

    /// Rotate active MemTable into immutable queue and allocate a fresh WAL
    fn rotate_active_memtable(&self) -> io::Result<()> {
        let mut active_lock = self.active_memtable.write();
        if active_lock.approximate_size() < self.options.memtable_size_bytes {
            return Ok(()); // Another thread rotated already
        }

        let old_mem = active_lock.clone();
        let new_mem_id = self.next_mem_id.fetch_add(1, Ordering::SeqCst) as usize;
        *active_lock = Arc::new(MemTable::new(new_mem_id));

        self.imm_memtables.write().push(old_mem);

        // Flush immutable tables to SSTables
        self.flush_immutable_memtables()?;

        Ok(())
    }

    /// Flush all queued immutable MemTables to Level 0 SSTables
    pub fn flush_immutable_memtables(&self) -> io::Result<()> {
        let mut imms_lock = self.imm_memtables.write();
        if imms_lock.is_empty() {
            return Ok(());
        }

        let mut compactor = self.compactor.lock();

        for imm in imms_lock.drain(..) {
            let entries = imm.iter_all();
            if entries.is_empty() {
                continue;
            }

            let sst_path = self.path.join(format!("L0_{:06}.sst", imm.id()));
            let mut builder = SsTableBuilder::create(&sst_path)?;

            for kv in &entries {
                builder.add(kv)?;
            }

            builder.finish()?;
            compactor.add_l0_table(sst_path);
        }

        // Trigger background compaction if L0 trigger exceeded
        if compactor.needs_compaction() {
            compactor.compact_l0_to_l1()?;
        }

        Ok(())
    }

    /// Force manual flush of active memtable to disk
    pub fn flush(&self) -> io::Result<()> {
        self.wal.lock().sync()?;
        let mut active_lock = self.active_memtable.write();
        let old_mem = active_lock.clone();
        let new_mem_id = self.next_mem_id.fetch_add(1, Ordering::SeqCst) as usize;
        *active_lock = Arc::new(MemTable::new(new_mem_id));

        self.imm_memtables.write().push(old_mem);
        drop(active_lock);

        self.flush_immutable_memtables()?;
        Ok(())
    }

    /// Force manual compaction
    pub fn compact(&self) -> io::Result<()> {
        let mut compactor = self.compactor.lock();
        compactor.compact_l0_to_l1()?;
        Ok(())
    }
}
