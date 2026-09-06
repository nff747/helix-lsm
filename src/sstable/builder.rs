use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};
use byteorder::{LittleEndian, WriteBytesExt};
use crc32fast::Hasher;

use crate::sstable::bloom::BloomFilter;
use crate::types::KeyValue;

pub const SSTABLE_MAGIC: u64 = 0x48454C4958535354; // "HELIXSST"
pub const TARGET_BLOCK_SIZE: usize = 4096; // 4KB uncompressed block target

#[derive(Debug, Clone)]
pub struct BlockMeta {
    pub offset: u64,
    pub length: u32,
    pub first_key: Vec<u8>,
}

pub struct SsTableBuilder {
    writer: BufWriter<File>,
    path: PathBuf,
    current_block: Vec<u8>,
    block_metas: Vec<BlockMeta>,
    keys_for_bloom: Vec<Vec<u8>>,
    current_offset: u64,
    first_key_in_block: Option<Vec<u8>>,
}

impl SsTableBuilder {
    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path.as_ref())?;

        Ok(Self {
            writer: BufWriter::with_capacity(256 * 1024, file),
            path: path.as_ref().to_path_buf(),
            current_block: Vec::with_capacity(TARGET_BLOCK_SIZE),
            block_metas: Vec::new(),
            keys_for_bloom: Vec::new(),
            current_offset: 0,
            first_key_in_block: None,
        })
    }

    /// Add a key-value record to the SSTable (assumed in strictly sorted order)
    pub fn add(&mut self, kv: &KeyValue) -> io::Result<()> {
        if self.first_key_in_block.is_none() {
            self.first_key_in_block = Some(kv.key.user_key.to_vec());
        }

        self.keys_for_bloom.push(kv.key.user_key.to_vec());

        // Serialize entry
        self.current_block.write_u64::<LittleEndian>(kv.key.seq_num)?;
        self.current_block.write_u8(kv.key.value_type as u8)?;
        self.current_block.write_u16::<LittleEndian>(kv.key.user_key.len() as u16)?;
        self.current_block.write_u32::<LittleEndian>(kv.value.len() as u32)?;
        self.current_block.write_all(&kv.key.user_key)?;
        self.current_block.write_all(&kv.value)?;

        if self.current_block.len() >= TARGET_BLOCK_SIZE {
            self.flush_current_block()?;
        }

        Ok(())
    }

    fn flush_current_block(&mut self) -> io::Result<()> {
        if self.current_block.is_empty() {
            return Ok(());
        }

        let block_len = self.current_block.len() as u32;
        self.writer.write_all(&self.current_block)?;

        self.block_metas.push(BlockMeta {
            offset: self.current_offset,
            length: block_len,
            first_key: self.first_key_in_block.take().unwrap_or_default(),
        });

        self.current_offset += block_len as u64;
        self.current_block.clear();

        Ok(())
    }

    /// Finalize SSTable layout: Data Blocks -> Index Block -> Bloom Filter -> 52B Footer
    pub fn finish(mut self) -> io::Result<PathBuf> {
        self.flush_current_block()?;

        // 1. Write Index Block
        let index_offset = self.current_offset;
        let mut index_buf = Vec::new();
        index_buf.write_u32::<LittleEndian>(self.block_metas.len() as u32)?;

        for meta in &self.block_metas {
            index_buf.write_u64::<LittleEndian>(meta.offset)?;
            index_buf.write_u32::<LittleEndian>(meta.length)?;
            index_buf.write_u16::<LittleEndian>(meta.first_key.len() as u16)?;
            index_buf.write_all(&meta.first_key)?;
        }

        let index_len = index_buf.len() as u64;
        self.writer.write_all(&index_buf)?;
        self.current_offset += index_len;

        // 2. Write Bloom Filter Block
        let bloom_offset = self.current_offset;
        let key_slices: Vec<&[u8]> = self.keys_for_bloom.iter().map(|k| k.as_slice()).collect();
        let bloom = BloomFilter::build(&key_slices, 10); // 10 bits per key (~1% FP rate)

        let mut bloom_buf = Vec::new();
        bloom_buf.write_u8(bloom.num_probes())?;
        bloom_buf.write_all(bloom.as_bytes())?;

        let bloom_len = bloom_buf.len() as u64;
        self.writer.write_all(&bloom_buf)?;
        self.current_offset += bloom_len;

        // 3. Write Footer with Checksum
        let mut footer = Vec::new();
        footer.write_u64::<LittleEndian>(index_offset)?;
        footer.write_u64::<LittleEndian>(index_len)?;
        footer.write_u64::<LittleEndian>(bloom_offset)?;
        footer.write_u64::<LittleEndian>(bloom_len)?;
        footer.write_u64::<LittleEndian>(SSTABLE_MAGIC)?;

        let mut hasher = Hasher::new();
        hasher.update(&footer);
        let checksum = hasher.finalize();
        footer.write_u32::<LittleEndian>(checksum)?;

        self.writer.write_all(&footer)?;
        self.writer.flush()?;

        Ok(self.path)
    }
}
