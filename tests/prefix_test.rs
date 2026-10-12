use helix_lsm::prefix_block::PrefixCompressor;

#[test]
fn test_prefix_compression_recovery() {
    let k1 = b"user:profile:alpha";
    let k2 = b"user:profile:bravo";
    let val = b"data_payload";

    let entry = PrefixCompressor::compress(k1, k2, val);
    assert_eq!(entry.shared_len, 13); // b"user:profile:"
    assert_eq!(entry.unshared_key, b"bravo");

    let restored = PrefixCompressor::decompress(k1, &entry);
    assert_eq!(restored, k2);
}
