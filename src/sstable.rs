//! Immutable Sorted String Table (SSTable) file writer and point-lookup reader.

use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use crate::bloom::BloomFilter;

pub const SSTABLE_MAGIC: u32 = 0x53535431; // 'SST1'

#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub key: Vec<u8>,
    pub offset: u64,
}

pub struct SsTableWriter;

impl SsTableWriter {
    /// Writes sorted key-values from MemTable to a new .sst file.
    pub fn create<P: AsRef<Path>>(
        path: P,
        entries: &[(Vec<u8>, Option<Vec<u8>>)],
    ) -> io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let mut writer = BufWriter::new(file);

        let mut bloom = BloomFilter::new(entries.len(), 10);
        let mut index = Vec::new();
        let mut current_offset: u64 = 0;

        // 1. Data Block
        for (i, (key, val)) in entries.iter().enumerate() {
            bloom.insert(key);

            // Sparse index: every 8 keys or first key
            if i % 8 == 0 {
                index.push(IndexEntry {
                    key: key.clone(),
                    offset: current_offset,
                });
            }

            let is_tombstone = val.is_none();
            let key_len = key.len() as u32;
            let val_bytes = val.as_deref().unwrap_or(&[]);
            let val_len = val_bytes.len() as u32;

            writer.write_all(&[if is_tombstone { 1u8 } else { 0u8 }])?;
            writer.write_all(&key_len.to_le_bytes())?;
            writer.write_all(key)?;
            writer.write_all(&val_len.to_le_bytes())?;
            writer.write_all(val_bytes)?;

            current_offset += 1 + 4 + key.len() as u64 + 4 + val_bytes.len() as u64;
        }

        // 2. Index Block
        let index_offset = current_offset;
        let index_len = index.len() as u32;
        writer.write_all(&index_len.to_le_bytes())?;
        current_offset += 4;

        for entry in &index {
            let k_len = entry.key.len() as u32;
            writer.write_all(&k_len.to_le_bytes())?;
            writer.write_all(&entry.key)?;
            writer.write_all(&entry.offset.to_le_bytes())?;
            current_offset += 4 + entry.key.len() as u64 + 8;
        }

        // 3. Bloom Filter Block
        let bloom_offset = current_offset;
        let bloom_bytes = bloom.as_bytes();
        let bloom_len = bloom_bytes.len() as u32;
        writer.write_all(&bloom_len.to_le_bytes())?;
        writer.write_all(&(bloom.num_hashes() as u32).to_le_bytes())?;
        writer.write_all(bloom_bytes)?;
        

        // 4. Footer: index_offset (8) + bloom_offset (8) + magic (4) = 20 bytes
        writer.write_all(&index_offset.to_le_bytes())?;
        writer.write_all(&bloom_offset.to_le_bytes())?;
        writer.write_all(&SSTABLE_MAGIC.to_le_bytes())?;

        writer.flush()?;
        Ok(())
    }
}

pub struct SsTableReader {
    path: PathBuf,
    bloom: BloomFilter,
    index: Vec<IndexEntry>,
}

impl SsTableReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let mut file = File::open(&path_buf)?;
        let file_len = file.metadata()?.len();

        if file_len < 20 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "SSTable file too small"));
        }

        // Read footer
        file.seek(SeekFrom::End(-20))?;
        let mut footer = [0u8; 20];
        file.read_exact(&mut footer)?;

        let index_offset = u64::from_le_bytes(footer[0..8].try_into().unwrap());
        let bloom_offset = u64::from_le_bytes(footer[8..16].try_into().unwrap());
        let magic = u32::from_le_bytes(footer[16..20].try_into().unwrap());

        if magic != SSTABLE_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid SSTable magic"));
        }

        // Read Index Block
        file.seek(SeekFrom::Start(index_offset))?;
        let mut idx_len_buf = [0u8; 4];
        file.read_exact(&mut idx_len_buf)?;
        let num_index_entries = u32::from_le_bytes(idx_len_buf) as usize;

        let mut index = Vec::with_capacity(num_index_entries);
        for _ in 0..num_index_entries {
            let mut klen_buf = [0u8; 4];
            file.read_exact(&mut klen_buf)?;
            let klen = u32::from_le_bytes(klen_buf) as usize;

            let mut k = vec![0u8; klen];
            file.read_exact(&mut k)?;

            let mut off_buf = [0u8; 8];
            file.read_exact(&mut off_buf)?;
            let offset = u64::from_le_bytes(off_buf);

            index.push(IndexEntry { key: k, offset });
        }

        // Read Bloom Filter Block
        file.seek(SeekFrom::Start(bloom_offset))?;
        let mut blen_buf = [0u8; 4];
        file.read_exact(&mut blen_buf)?;
        let blen = u32::from_le_bytes(blen_buf) as usize;

        let mut nhash_buf = [0u8; 4];
        file.read_exact(&mut nhash_buf)?;
        let nhash = u32::from_le_bytes(nhash_buf) as usize;

        let mut bbytes = vec![0u8; blen];
        file.read_exact(&mut bbytes)?;
        let bloom = BloomFilter::from_bytes(bbytes, nhash);

        Ok(Self {
            path: path_buf,
            bloom,
            index,
        })
    }

    /// Point lookup: checks Bloom filter -> Binary search index -> Reads block
    pub fn get(&self, target_key: &[u8]) -> io::Result<Option<Option<Vec<u8>>>> {
        if !self.bloom.may_contain(target_key) {
            return Ok(None); // Not in SSTable
        }

        if self.index.is_empty() {
            return Ok(None);
        }

        // Binary search index to find candidate starting offset
        let mut low = 0;
        let mut high = self.index.len();
        while low + 1 < high {
            let mid = low + (high - low) / 2;
            if self.index[mid].key.as_slice() <= target_key {
                low = mid;
            } else {
                high = mid;
            }
        }

        let start_offset = self.index[low].offset;
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(start_offset))?;

        // Scan entries until key matches or exceeds
        loop {
            let mut tombstone_buf = [0u8; 1];
            if file.read_exact(&mut tombstone_buf).is_err() {
                break; // EOF
            }
            let is_tombstone = tombstone_buf[0] == 1;

            let mut klen_buf = [0u8; 4];
            file.read_exact(&mut klen_buf)?;
            let klen = u32::from_le_bytes(klen_buf) as usize;

            let mut key = vec![0u8; klen];
            file.read_exact(&mut key)?;

            let mut vlen_buf = [0u8; 4];
            file.read_exact(&mut vlen_buf)?;
            let vlen = u32::from_le_bytes(vlen_buf) as usize;

            let mut val = vec![0u8; vlen];
            file.read_exact(&mut val)?;

            if key.as_slice() == target_key {
                if is_tombstone {
                    return Ok(Some(None)); // Deleted tombstone
                } else {
                    return Ok(Some(Some(val))); // Found value
                }
            } else if key.as_slice() > target_key {
                break; // Passed target key in sorted order
            }
        }

        Ok(None)
    }

    pub fn read_all_entries(&self) -> io::Result<Vec<(Vec<u8>, Option<Vec<u8>>)>> {
        let mut file = File::open(&self.path)?;
        // Stop before footer and index
        file.seek(SeekFrom::End(-20))?;
        let mut footer = [0u8; 20];
        file.read_exact(&mut footer)?;
        let index_offset = u64::from_le_bytes(footer[0..8].try_into().unwrap());

        file.seek(SeekFrom::Start(0))?;
        let mut current: u64 = 0;
        let mut entries = Vec::new();

        while current < index_offset {
            let mut tombstone_buf = [0u8; 1];
            if file.read_exact(&mut tombstone_buf).is_err() {
                break;
            }
            let is_tombstone = tombstone_buf[0] == 1;

            let mut klen_buf = [0u8; 4];
            file.read_exact(&mut klen_buf)?;
            let klen = u32::from_le_bytes(klen_buf) as usize;

            let mut key = vec![0u8; klen];
            file.read_exact(&mut key)?;

            let mut vlen_buf = [0u8; 4];
            file.read_exact(&mut vlen_buf)?;
            let vlen = u32::from_le_bytes(vlen_buf) as usize;

            let mut val = vec![0u8; vlen];
            file.read_exact(&mut val)?;

            current += 1 + 4 + klen as u64 + 4 + vlen as u64;

            entries.push((key, if is_tombstone { None } else { Some(val) }));
        }

        Ok(entries)
    }
}
