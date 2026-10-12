//! Encapsulates raw data block payload with trailing CRC32 checksum.

use crate::crc::Crc32;

pub struct ChecksummedDataBlock;

impl ChecksummedDataBlock {
    pub fn build(payload: &[u8]) -> Vec<u8> {
        let checksum = Crc32::checksum(payload);
        let mut block = Vec::with_capacity(payload.len() + 4);
        block.extend_from_slice(payload);
        block.extend_from_slice(&checksum.to_le_bytes());
        block
    }

    pub fn verify_and_unwrap(block: &[u8]) -> Option<&[u8]> {
        if block.len() < 4 {
            return None;
        }
        let payload = &block[..block.len() - 4];
        let mut cs_bytes = [0u8; 4];
        cs_bytes.copy_from_slice(&block[block.len() - 4..]);
        let expected_checksum = u32::from_le_bytes(cs_bytes);

        if Crc32::checksum(payload) == expected_checksum {
            Some(payload)
        } else {
            None
        }
    }
}
