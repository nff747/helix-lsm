use helix_lsm::sstable_meta::{TableFooter, SSTABLE_MAGIC};

#[test]
fn test_footer_encode_decode() {
    let footer = TableFooter {
        index_offset: 1024,
        index_len: 256,
        bloom_offset: 2048,
        magic: SSTABLE_MAGIC,
    };
    let bytes = footer.encode();
    let decoded = TableFooter::decode(&bytes).expect("failed to decode footer");
    assert_eq!(decoded, footer);
}

#[test]
fn test_footer_invalid_magic() {
    let footer = TableFooter {
        index_offset: 0,
        index_len: 0,
        bloom_offset: 0,
        magic: 0xDEADBEEF,
    };
    let bytes = footer.encode();
    assert!(TableFooter::decode(&bytes).is_none());
}
