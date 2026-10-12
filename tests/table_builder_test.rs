use helix_lsm::table_builder::TableBuilder;

#[test]
fn test_table_builder_rollover() {
    let mut builder = TableBuilder::new(100);
    // Add small entry
    builder.add(b"key1", &vec![0u8; 40]);
    assert_eq!(builder.index.entries.len(), 0);

    // Add entry that triggers rollover
    builder.add(b"key2", &vec![0u8; 70]);
    assert_eq!(builder.index.entries.len(), 1);
    assert_eq!(builder.index.entries[0].key, b"key2");
}
