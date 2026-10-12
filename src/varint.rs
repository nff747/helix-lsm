//! Variable-length integer (Varint) encoder and decoder for compact SSTable block indexing.

pub struct Varint;

impl Varint {
    pub fn encode_u64(mut val: u64, buf: &mut Vec<u8>) {
        while val >= 0x80 {
            buf.push(((val & 0x7F) as u8) | 0x80);
            val >>= 7;
        }
        buf.push(val as u8);
    }

    pub fn decode_u64(bytes: &[u8]) -> Option<(u64, usize)> {
        let mut res = 0u64;
        let mut shift = 0;
        for (i, &b) in bytes.iter().enumerate() {
            res |= ((b & 0x7F) as u64) << shift;
            if (b & 0x80) == 0 {
                return Some((res, i + 1));
            }
            shift += 7;
            if shift > 64 {
                return None;
            }
        }
        None
    }
}
