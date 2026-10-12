//! IEEE 802.3 CRC32 checksum engine for block and WAL corruption verification.

pub struct Crc32;

impl Crc32 {
    const TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut c = i as u32;
            let mut j = 0;
            while j < 8 {
                if (c & 1) != 0 {
                    c = 0xEDB88320 ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                j += 1;
            }
            table[i] = c;
            i += 1;
        }
        table
    };

    pub fn checksum(data: &[u8]) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        for &b in data {
            let idx = ((crc ^ (b as u32)) & 0xFF) as usize;
            crc = Self::TABLE[idx] ^ (crc >> 8);
        }
        !crc
    }
}
