//! Prefix key compression: eliminates common key prefixes in SSTable data blocks.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixEntry {
    pub shared_len: usize,
    pub unshared_key: Vec<u8>,
    pub value: Vec<u8>,
}

pub struct PrefixCompressor;

impl PrefixCompressor {
    pub fn compress(prev_key: &[u8], curr_key: &[u8], value: &[u8]) -> PrefixEntry {
        let mut shared_len = 0;
        let max_shared = prev_key.len().min(curr_key.len());
        while shared_len < max_shared && prev_key[shared_len] == curr_key[shared_len] {
            shared_len += 1;
        }

        PrefixEntry {
            shared_len,
            unshared_key: curr_key[shared_len..].to_vec(),
            value: value.to_vec(),
        }
    }

    pub fn decompress(prev_key: &[u8], entry: &PrefixEntry) -> Vec<u8> {
        let mut key = Vec::with_capacity(entry.shared_len + entry.unshared_key.len());
        key.extend_from_slice(&prev_key[..entry.shared_len]);
        key.extend_from_slice(&entry.unshared_key);
        key
    }
}
