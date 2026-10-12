use helix_lsm::counting_bloom::CountingBloomFilter;

#[test]
fn test_counting_bloom_insertion_and_deletion() {
    let mut bloom = CountingBloomFilter::new(1024, 3);
    assert_eq!(bloom.contains(b"alpha"), false);

    bloom.insert(b"alpha");
    assert_eq!(bloom.contains(b"alpha"), true);

    bloom.remove(b"alpha");
    assert_eq!(bloom.contains(b"alpha"), false);
}
