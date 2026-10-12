use helix_lsm::merge_iter::KeyValue;
use helix_lsm::range_scan::RangeScanIterator;

#[test]
fn test_range_scan_bounded() {
    let data = vec![
        KeyValue { key: b"key:1".to_vec(), value: Some(b"v1".to_vec()) },
        KeyValue { key: b"key:2".to_vec(), value: Some(b"v2".to_vec()) },
        KeyValue { key: b"key:3".to_vec(), value: Some(b"v3".to_vec()) },
        KeyValue { key: b"key:4".to_vec(), value: Some(b"v4".to_vec()) },
    ];

    let scan = RangeScanIterator::new(data.into_iter(), Some(b"key:3".to_vec()));
    let keys: Vec<Vec<u8>> = scan.map(|kv| kv.key).collect();
    assert_eq!(keys, vec![b"key:1".to_vec(), b"key:2".to_vec()]);
}
