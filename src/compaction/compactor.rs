use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use bytes::Bytes;

use crate::sstable::{SsTableBuilder, SsTableReader};
use crate::types::{KeyValue, ValueType};

pub const L0_COMPACTION_TRIGGER: usize = 4; // Compact when L0 reaches 4 tables
pub const MAX_LEVELS: usize = 7;

#[derive(Debug, Clone, Default)]
pub struct LevelManifest {
    pub levels: Vec<Vec<PathBuf>>,
}

impl LevelManifest {
    pub fn new() -> Self {
        Self {
            levels: vec![Vec::new(); MAX_LEVELS],
        }
    }
}

/// Helper struct for K-way merge iterator across multiple SSTables
struct MergeIteratorEntry {
    kv: KeyValue,
    source_idx: usize,
}

impl PartialEq for MergeIteratorEntry {
    fn eq(&self, other: &Self) -> bool {
        self.kv.key == other.kv.key
    }
}

impl Eq for MergeIteratorEntry {}

impl Ord for MergeIteratorEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap acts as a Min-Heap
        other.kv.key.cmp(&self.kv.key)
    }
}

impl PartialOrd for MergeIteratorEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct CompactionManager {
    db_path: PathBuf,
    manifest: LevelManifest,
    next_sst_id: usize,
}

impl CompactionManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
            manifest: LevelManifest::new(),
            next_sst_id: 1,
        }
    }

    pub fn manifest(&self) -> &LevelManifest {
        &self.manifest
    }

    pub fn add_l0_table(&mut self, sst_path: PathBuf) {
        self.manifest.levels[0].push(sst_path);
    }

    pub fn needs_compaction(&self) -> bool {
        self.manifest.levels[0].len() >= L0_COMPACTION_TRIGGER
    }

    /// Execute Level-0 to Level-1 compaction routine.
    pub fn compact_l0_to_l1(&mut self) -> io::Result<()> {
        self.compact_level(0)
    }

    /// Generalized multi-level compaction routine from level `L_i` to `L_{i+1}`.
    /// Merges overlapping SSTables, dedupes MVCC versions, and purges obsolete tombstones
    /// when cascading to bottom levels.
    pub fn compact_level(&mut self, source_level: usize) -> io::Result<()> {
        if source_level >= MAX_LEVELS - 1 {
            return Ok(());
        }
        let target_level = source_level + 1;

        if self.manifest.levels[source_level].is_empty() {
            return Ok(());
        }

        let source_files = self.manifest.levels[source_level].clone();
        let target_files = self.manifest.levels[target_level].clone();

        // 1. Initialize streaming iterators from source and target tables
        let mut source_iterators = Vec::new();
        let mut all_files_to_remove = Vec::new();

        for path in source_files.iter().chain(target_files.iter()) {
            if let Ok(reader) = SsTableReader::open(path) {
                let stream = reader.into_stream()?;
                source_iterators.push(stream);
                all_files_to_remove.push(path.clone());
            }
        }

        // 2. Streaming K-Way Merge using Min-Heap
        let mut heap: BinaryHeap<MergeIteratorEntry> = BinaryHeap::new();
        for (idx, iter) in source_iterators.iter_mut().enumerate() {
            if let Some(Ok(kv)) = iter.next() {
                heap.push(MergeIteratorEntry {
                    kv,
                    source_idx: idx,
                });
            }
        }

        let new_sst_path = self.db_path.join(format!("L{}_{:06}.sst", target_level, self.next_sst_id));
        self.next_sst_id += 1;
        let mut builder = SsTableBuilder::create(&new_sst_path)?;

        let mut last_user_key: Option<Bytes> = None;
        let mut compacted_count = 0;

        let at_bottom_level = target_level == MAX_LEVELS - 1
            || self.manifest.levels[(target_level + 1)..].iter().all(|l| l.is_empty());

        while let Some(top) = heap.pop() {
            let current_user_key = top.kv.key.user_key.clone();

            // Deduplication: Only keep the newest version for each user key
            let is_newest_version = match &last_user_key {
                Some(prev) => prev != &current_user_key,
                None => true,
            };

            if is_newest_version {
                last_user_key = Some(current_user_key.clone());

                // If tombstone at the bottom level, purge it to reclaim disk space
                let is_tombstone = top.kv.key.value_type == ValueType::Deletion;

                if !(is_tombstone && at_bottom_level) {
                    builder.add(&top.kv)?;
                    compacted_count += 1;
                }
            }

            // Refill heap from streaming source iterator
            if let Some(Ok(next_kv)) = source_iterators[top.source_idx].next() {
                heap.push(MergeIteratorEntry {
                    kv: next_kv,
                    source_idx: top.source_idx,
                });
            }
        }

        if compacted_count > 0 {
            builder.finish()?;
            self.manifest.levels[target_level] = vec![new_sst_path];
        } else {
            // All keys were purged tombstones
            let _ = fs::remove_file(&new_sst_path);
            self.manifest.levels[target_level].clear();
        }

        // 3. Clear source level and delete old files
        self.manifest.levels[source_level].clear();
        for old_file in all_files_to_remove {
            let _ = fs::remove_file(old_file);
        }

        Ok(())
    }

    /// Automatically cascades compaction down through all levels where thresholds are exceeded.
    pub fn run_cascading_compaction(&mut self) -> io::Result<()> {
        for level in 0..(MAX_LEVELS - 1) {
            let threshold = if level == 0 { L0_COMPACTION_TRIGGER } else { 2 };
            if self.manifest.levels[level].len() >= threshold {
                self.compact_level(level)?;
            }
        }
        Ok(())
    }
}
