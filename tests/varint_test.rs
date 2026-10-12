use helix_lsm::varint::Varint;

#[test]
fn test_varint_roundtrip() {
    let test_values = [0, 1, 127, 128, 300, 16384, u64::MAX];
    let mut buf = Vec::new();

    for &v in &test_values {
        buf.clear();
        Varint::encode_u64(v, &mut buf);
        let (decoded, bytes_read) = Varint::decode_u64(&buf).expect("failed to decode");
        assert_eq!(decoded, v);
        assert_eq!(bytes_read, buf.len());
    }
}
