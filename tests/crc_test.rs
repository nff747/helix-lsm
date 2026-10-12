use helix_lsm::crc::Crc32;

#[test]
fn test_crc32_standard() {
    assert_eq!(Crc32::checksum(b"123456789"), 0xCBF43926);
    assert_eq!(Crc32::checksum(b""), 0x00000000);
}

#[test]
fn test_crc32_corruption_detection() {
    let original = b"immutable log structured merge block";
    let cs = Crc32::checksum(original);

    let mut corrupted = original.to_vec();
    corrupted[5] ^= 0xFF;
    assert_ne!(Crc32::checksum(&corrupted), cs);
}
