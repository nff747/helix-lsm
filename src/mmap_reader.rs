//! Zero-copy memory mapped byte slice reader for SSTable files.

pub struct MmapSliceReader<'a> {
    data: &'a [u8],
}

impl<'a> MmapSliceReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn read_exact_at(&self, offset: usize, len: usize) -> Option<&'a [u8]> {
        if offset + len <= self.data.len() {
            Some(&self.data[offset..offset + len])
        } else {
            None
        }
    }

    pub fn total_len(&self) -> usize {
        self.data.len()
    }
}
