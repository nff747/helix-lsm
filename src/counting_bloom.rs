//! Counting Bloom Filter: 4-bit counters per bucket to allow removals.

pub struct CountingBloomFilter {
    counters: Vec<u8>,
    num_hashes: usize,
}

impl CountingBloomFilter {
    pub fn new(size: usize, num_hashes: usize) -> Self {
        Self {
            counters: vec![0; size],
            num_hashes,
        }
    }

    fn hashes(&self, key: &[u8]) -> Vec<usize> {
        let mut hashes = Vec::with_capacity(self.num_hashes);
        let mut h1 = 0xcbf29ce484222325u64;
        for &b in key {
            h1 ^= b as u64;
            h1 = h1.wrapping_mul(0x100000001b3);
        }
        for i in 0..self.num_hashes {
            let h = h1.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            hashes.push((h as usize) % self.counters.len());
        }
        hashes
    }

    pub fn insert(&mut self, key: &[u8]) {
        for idx in self.hashes(key) {
            if self.counters[idx] < 15 {
                self.counters[idx] += 1;
            }
        }
    }

    pub fn remove(&mut self, key: &[u8]) {
        for idx in self.hashes(key) {
            if self.counters[idx] > 0 {
                self.counters[idx] -= 1;
            }
        }
    }

    pub fn contains(&self, key: &[u8]) -> bool {
        for idx in self.hashes(key) {
            if self.counters[idx] == 0 {
                return false;
            }
        }
        true
    }
}
