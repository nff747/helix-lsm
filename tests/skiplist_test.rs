use helix_lsm::skiplist_memtable::SkiplistMemtable;

#[test]
fn test_skiplist_memtable_basic() {
    let mem = SkiplistMemtable::new();
    mem.put(b"alpha", b"123");
    mem.put(b"beta", b"456");

    assert_eq!(mem.get(b"alpha").unwrap(), Some(b"123".to_vec()));
    assert_eq!(mem.get(b"gamma"), None);
    assert!(mem.size_bytes() > 0);
}
