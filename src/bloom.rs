//! High-performance Bloom Filter for fast negative key lookups in SSTables.

#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<u8>,
    num_bits: usize,
    num_hashes: usize,
}

impl BloomFilter {
    pub fn new(num_keys: usize, bits_per_key: usize) -> Self {
        let num_bits = std::cmp::max(64, num_keys * bits_per_key);
        let num_bytes = (num_bits + 7) / 8;
        let num_hashes = std::cmp::max(1, (bits_per_key as f64 * 0.693).round() as usize);
        Self {
            bits: vec![0u8; num_bytes],
            num_bits,
            num_hashes,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, num_hashes: usize) -> Self {
        let num_bits = bytes.len() * 8;
        Self {
            bits: bytes,
            num_bits,
            num_hashes,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }

    pub fn num_hashes(&self) -> usize {
        this_or(self.num_hashes)
    }

    pub fn insert(&mut self, key: &[u8]) {
        let (h1, h2) = Self::hash(key);
        for i in 0..self.num_hashes {
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let bit_idx = (combined % self.num_bits as u64) as usize;
            self.bits[bit_idx / 8] |= 1 << (bit_idx % 8);
        }
    }

    pub fn may_contain(&self, key: &[u8]) -> bool {
        if self.num_bits == 0 {
            return false;
        }
        let (h1, h2) = Self::hash(key);
        for i in 0..self.num_hashes {
            let combined = h1.wrapping_add((i as u64).wrapping_mul(h2));
            let bit_idx = (combined % self.num_bits as u64) as usize;
            if (self.bits[bit_idx / 8] & (1 << (bit_idx % 8))) == 0 {
                return false;
            }
        }
        true
    }

    /// FNV-1a derived double hashing
    fn hash(key: &[u8]) -> (u64, u64) {
        let mut h1: u64 = 0xcbf29ce484222325;
        let mut h2: u64 = 0x100000001b3;
        for &byte in key {
            h1 ^= byte as u64;
            h1 = h1.wrapping_mul(0x100000001b3);
            h2 = h2.rotate_left(5) ^ (byte as u64);
        }
        (h1, h2 | 1)
    }
}

fn this_or(val: usize) -> usize {
    val
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_membership() {
        let mut filter = BloomFilter::new(100, 10);
        filter.insert(b"user_123");
        filter.insert(b"user_456");

        assert!(filter.may_contain(b"user_123"));
        assert!(filter.may_contain(b"user_456"));
        assert!(!filter.may_contain(b"user_999"));
    }
}
