//! SSTable format footer metadata.

pub const SSTABLE_MAGIC: u64 = 0x48454C49584C534D; // HELIXLSM
pub const FOOTER_ENCODED_SIZE: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableFooter {
    pub index_offset: u64,
    pub index_len: u64,
    pub bloom_offset: u64,
    pub magic: u64,
}

impl TableFooter {
    pub fn encode(&self) -> [u8; FOOTER_ENCODED_SIZE] {
        let mut buf = [0u8; FOOTER_ENCODED_SIZE];
        buf[0..8].copy_from_slice(&self.index_offset.to_le_bytes());
        buf[8..16].copy_from_slice(&self.index_len.to_le_bytes());
        buf[16..24].copy_from_slice(&self.bloom_offset.to_le_bytes());
        buf[24..32].copy_from_slice(&self.magic.to_le_bytes());
        buf
    }

    pub fn decode(bytes: &[u8; FOOTER_ENCODED_SIZE]) -> Option<Self> {
        let mut o = [0u8; 8];
        o.copy_from_slice(&bytes[0..8]);
        let index_offset = u64::from_le_bytes(o);

        o.copy_from_slice(&bytes[8..16]);
        let index_len = u64::from_le_bytes(o);

        o.copy_from_slice(&bytes[16..24]);
        let bloom_offset = u64::from_le_bytes(o);

        o.copy_from_slice(&bytes[24..32]);
        let magic = u64::from_le_bytes(o);

        if magic != SSTABLE_MAGIC {
            return None;
        }

        Some(Self {
            index_offset,
            index_len,
            bloom_offset,
            magic,
        })
    }
}
