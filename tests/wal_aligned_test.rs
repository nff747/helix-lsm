use helix_lsm::wal_aligned::{PageAlignedWalBuffer, PAGE_SIZE};

#[test]
fn test_aligned_buffer_append() {
    let mut buf = PageAlignedWalBuffer::new(1);
    assert_eq!(buf.append(b"hello world"), true);
    assert_eq!(buf.flushed_bytes(), b"hello world");

    let big = vec![0xFFu8; PAGE_SIZE];
    assert_eq!(buf.append(&big), false); // Cannot exceed page capacity
}
