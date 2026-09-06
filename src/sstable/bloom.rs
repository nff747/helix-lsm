/// Cache-efficient Bloom Filter implementation using double hashing.
/// k = (m/n) * ln(2) ~= 7 for 10 bits per key (~1% false positive rate).
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<u8>,
    num_probes: u8,
}

impl BloomFilter {
    pub fn build(keys: &[&[u8]], bits_per_key: usize) -> Self {
        let n = keys.len().max(1);
        let mut num_bits = n * bits_per_key;
        num_bits = ((num_bits + 7) / 8) * 8; // Align to bytes
        num_bits = num_bits.max(64);

        let num_probes = ((bits_per_key as f64) * 0.69).round().clamp(1.0, 30.0) as u8;
        let mut bits = vec![0u8; num_bits / 8];

        for key in keys {
            let mut h = Self::hash(key);
            let delta = (h >> 17) | (h << 15); // Rotate for second hash probe
            for _ in 0..num_probes {
                let bit_pos = (h as usize) % num_bits;
                bits[bit_pos / 8] |= 1 << (bit_pos % 8);
                h = h.wrapping_add(delta);
            }
        }

        Self { bits, num_probes }
    }

    pub fn from_bytes(bytes: Vec<u8>, num_probes: u8) -> Self {
        Self { bits: bytes, num_probes }
    }

    pub fn may_contain(&self, key: &[u8]) -> bool {
        if self.bits.is_empty() {
            return false;
        }

        let num_bits = self.bits.len() * 8;
        let mut h = Self::hash(key);
        let delta = (h >> 17) | (h << 15);

        for _ in 0..self.num_probes {
            let bit_pos = (h as usize) % num_bits;
            if (self.bits[bit_pos / 8] & (1 << (bit_pos % 8))) == 0 {
                return false; // Definitely does not contain
            }
            h = h.wrapping_add(delta);
        }

        true // Might contain
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bits
    }

    pub fn num_probes(&self) -> u8 {
        self.num_probes
    }

    /// Fast 32-bit Murmur-style hash
    fn hash(data: &[u8]) -> u32 {
        let mut h: u32 = 0xbc9f1d34;
        for &byte in data {
            h = h.wrapping_mul(0x5bd1e995) ^ (byte as u32);
            h = (h << 13) | (h >> 19);
        }
        h
    }
}
