use helix_lsm::data_block::ChecksummedDataBlock;

#[test]
fn test_checksummed_block_roundtrip() {
    let raw = b"record_payload_entry_0123456789";
    let block = ChecksummedDataBlock::build(raw);
    assert_eq!(block.len(), raw.len() + 4);

    let extracted = ChecksummedDataBlock::verify_and_unwrap(&block).expect("checksum valid");
    assert_eq!(extracted, raw);

    let mut corrupted = block.clone();
    corrupted[0] ^= 0x01;
    assert!(ChecksummedDataBlock::verify_and_unwrap(&corrupted).is_none());
}
