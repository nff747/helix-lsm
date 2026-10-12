//! 4096-byte page aligned WAL buffer for O_DIRECT filesystem writes.

pub const PAGE_SIZE: usize = 4096;

pub struct PageAlignedWalBuffer {
    buffer: Vec<u8>,
    cursor: usize,
}

impl PageAlignedWalBuffer {
    pub fn new(capacity_pages: usize) -> Self {
        Self {
            buffer: vec![0u8; capacity_pages * PAGE_SIZE],
            cursor: 0,
        }
    }

    pub fn append(&mut self, data: &[u8]) -> bool {
        if self.cursor + data.len() > self.buffer.len() {
            return false;
        }
        self.buffer[self.cursor..self.cursor + data.len()].copy_from_slice(data);
        self.cursor += data.len();
        true
    }

    pub fn flushed_bytes(&self) -> &[u8] {
        &self.buffer[..self.cursor]
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }
}
