use helix_lsm::mmap_reader::MmapSliceReader;

#[test]
fn test_mmap_reader_slice_bounds() {
    let raw = b"abcdefghijklmnopqrstuvwxyz";
    let reader = MmapSliceReader::new(raw);
    assert_eq!(reader.total_len(), 26);

    let sub = reader.read_exact_at(10, 4).unwrap();
    assert_eq!(sub, b"klmn");

    assert!(reader.read_exact_at(25, 5).is_none());
}
