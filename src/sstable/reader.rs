use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use byteorder::{LittleEndian, ReadBytesExt};
use bytes::Bytes;
use crc32fast::Hasher;

use crate::sstable::bloom::BloomFilter;
use crate::sstable::builder::{BlockMeta, SSTABLE_MAGIC};
use crate::types::{InternalKey, KeyValue, ValueType};

pub const FOOTER_SIZE: u64 = 8 + 8 + 8 + 8 + 8 + 4; // IndexOff(8) + IndexLen(8) + BloomOff(8) + BloomLen(8) + Magic(8) + CRC(4) = 44 bytes

pub struct SsTableReader {
    path: PathBuf,
    file: File,
    block_metas: Vec<BlockMeta>,
    bloom: BloomFilter,
    smallest_key: Vec<u8>,
    largest_key: Vec<u8>,
}

impl SsTableReader {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path.as_ref())?;
        let file_len = file.metadata()?.len();

        if file_len < FOOTER_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "SSTable file too small"));
        }

        // Read and verify Footer
        file.seek(SeekFrom::Start(file_len - FOOTER_SIZE))?;
        let mut footer_buf = vec![0u8; (FOOTER_SIZE - 4) as usize];
        file.read_exact(&mut footer_buf)?;
        let expected_checksum = file.read_u32::<LittleEndian>()?;

        let mut hasher = Hasher::new();
        hasher.update(&footer_buf);
        if hasher.finalize() != expected_checksum {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "SSTable footer CRC mismatch"));
        }

        let mut footer_cursor = io::Cursor::new(footer_buf);
        let index_offset = footer_cursor.read_u64::<LittleEndian>()?;
        let _index_len = footer_cursor.read_u64::<LittleEndian>()?;
        let bloom_offset = footer_cursor.read_u64::<LittleEndian>()?;
        let bloom_len = footer_cursor.read_u64::<LittleEndian>()?;
        let magic = footer_cursor.read_u64::<LittleEndian>()?;

        if magic != SSTABLE_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid SSTable magic header"));
        }

        // Load Bloom Filter
        file.seek(SeekFrom::Start(bloom_offset))?;
        let num_probes = file.read_u8()?;
        let mut bloom_bytes = vec![0u8; (bloom_len - 1) as usize];
        file.read_exact(&mut bloom_bytes)?;
        let bloom = BloomFilter::from_bytes(bloom_bytes, num_probes);

        // Load Sparse Index
        file.seek(SeekFrom::Start(index_offset))?;
        let num_blocks = file.read_u32::<LittleEndian>()? as usize;
        let mut block_metas = Vec::with_capacity(num_blocks);

        for _ in 0..num_blocks {
            let offset = file.read_u64::<LittleEndian>()?;
            let length = file.read_u32::<LittleEndian>()?;
            let key_len = file.read_u16::<LittleEndian>()? as usize;
            let mut key_buf = vec![0u8; key_len];
            file.read_exact(&mut key_buf)?;

            block_metas.push(BlockMeta {
                offset,
                length,
                first_key: key_buf,
            });
        }

        let smallest_key = block_metas.first().map(|b| b.first_key.clone()).unwrap_or_default();
        let largest_key = block_metas.last().map(|b| b.first_key.clone()).unwrap_or_default();

        Ok(Self {
            path: path.as_ref().to_path_buf(),
            file,
            block_metas,
            bloom,
            smallest_key,
            largest_key,
        })
    }

    /// Fast lookup: Bloom Filter check -> Binary Search Index -> Read Targeted 4KB Block
    pub fn get(&mut self, user_key: &[u8], snapshot_seq: u64) -> io::Result<Option<Option<Bytes>>> {
        // 1. O(1) Bloom filter test: rejects 99%+ of non-existent queries without disk seek
        if !self.bloom.may_contain(user_key) {
            return Ok(None);
        }

        if self.block_metas.is_empty() {
            return Ok(None);
        }

        // 2. Binary search sparse index to locate target block
        let block_idx = match self.block_metas.binary_search_by(|b| b.first_key.as_slice().cmp(user_key)) {
            Ok(idx) => idx,
            Err(0) => 0,
            Err(idx) => idx - 1,
        };

        let meta = &self.block_metas[block_idx];

        // 3. Read specific block from disk
        self.file.seek(SeekFrom::Start(meta.offset))?;
        let mut block_buf = vec![0u8; meta.length as usize];
        self.file.read_exact(&mut block_buf)?;

        let mut cursor = io::Cursor::new(block_buf);
        while (cursor.position() as usize) < meta.length as usize {
            let seq_num = cursor.read_u64::<LittleEndian>()?;
            let op_type_byte = cursor.read_u8()?;
            let key_len = cursor.read_u16::<LittleEndian>()? as usize;
            let val_len = cursor.read_u32::<LittleEndian>()? as usize;

            let mut k_buf = vec![0u8; key_len];
            cursor.read_exact(&mut k_buf)?;

            let mut v_buf = vec![0u8; val_len];
            cursor.read_exact(&mut v_buf)?;

            if k_buf.as_slice() == user_key && seq_num <= snapshot_seq {
                let value_type = ValueType::from(op_type_byte);
                return match value_type {
                    ValueType::Value => Ok(Some(Some(Bytes::from(v_buf)))),
                    ValueType::Deletion => Ok(Some(None)), // Explicit tombstone
                };
            }
        }

        Ok(None)
    }

    /// Sequential iteration through all entries (used during Compaction)
    pub fn iter_all(&mut self) -> io::Result<Vec<KeyValue>> {
        let mut results = Vec::new();

        for meta in &self.block_metas {
            self.file.seek(SeekFrom::Start(meta.offset))?;
            let mut block_buf = vec![0u8; meta.length as usize];
            self.file.read_exact(&mut block_buf)?;

            let mut cursor = io::Cursor::new(block_buf);
            while (cursor.position() as usize) < meta.length as usize {
                let seq_num = cursor.read_u64::<LittleEndian>()?;
                let op_type = cursor.read_u8()?;
                let key_len = cursor.read_u16::<LittleEndian>()? as usize;
                let val_len = cursor.read_u32::<LittleEndian>()? as usize;

                let mut k_buf = vec![0u8; key_len];
                cursor.read_exact(&mut k_buf)?;

                let mut v_buf = vec![0u8; val_len];
                cursor.read_exact(&mut v_buf)?;

                results.push(KeyValue {
                    key: InternalKey::new(Bytes::from(k_buf), seq_num, ValueType::from(op_type)),
                    value: Bytes::from(v_buf),
                });
            }
        }

        Ok(results)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn smallest_key(&self) -> &[u8] {
        &self.smallest_key
    }

    pub fn largest_key(&self) -> &[u8] {
        &self.largest_key
    }
}
