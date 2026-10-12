//! MVCC Snapshot Isolation: point-in-time consistent reads.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceNumber(pub u64);

pub struct Snapshot {
    pub seq: SequenceNumber,
}

impl Snapshot {
    pub fn new(seq: u64) -> Self {
        Self {
            seq: SequenceNumber(seq),
        }
    }

    pub fn is_visible(&self, write_seq: SequenceNumber) -> bool {
        write_seq <= self.seq
    }
}
