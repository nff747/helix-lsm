//! Compaction engine: merges overlapping SSTables, purges tombstones and duplicate versions.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use crate::sstable::{SsTableReader, SsTableWriter};

pub struct Compactor;

impl Compactor {
    /// Merges multiple SSTables into a single new SSTable file, returning the new SSTable path.
    pub fn compact<P: AsRef<Path>>(
        sstable_paths: &[PathBuf],
        output_path: P,
    ) -> io::Result<()> {
        let mut merged: BTreeMap<Vec<u8>, Option<Vec<u8>>> = BTreeMap::new();

        // Process from oldest to newest so newer keys overwrite older keys
        for path in sstable_paths {
            let reader = SsTableReader::open(path)?;
            let entries = reader.read_all_entries()?;
            for (key, val) in entries {
                merged.insert(key, val);
            }
        }

        // At the bottom level (full compaction), we can remove tombstones completely
        let final_entries: Vec<(Vec<u8>, Option<Vec<u8>>)> = merged
            .into_iter()
            .filter(|(_, val)| val.is_some())
            .collect();

        if !final_entries.is_empty() {
            SsTableWriter::create(output_path, &final_entries)?;
        }

        // Clean up compacted input files
        for path in sstable_paths {
            let _ = fs::remove_file(path);
        }

        Ok(())
    }
}
