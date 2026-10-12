use helix_lsm::block_index::BlockIndex;

#[test]
fn test_block_index_lookup() {
    let mut index = BlockIndex::new();
    index.add_entry(b"apple".to_vec(), 0, 100);
    index.add_entry(b"banana".to_vec(), 100, 100);
    index.add_entry(b"cherry".to_vec(), 200, 100);

    let handle = index.find_block(b"avocado").unwrap();
    assert_eq!(handle.offset, 100); // Should point to banana block

    let handle = index.find_block(b"apple").unwrap();
    assert_eq!(handle.offset, 0);
}
