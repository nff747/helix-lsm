use helix_lsm::merge_iter::{KeyValue, MergeIterator};

#[test]
fn test_merge_three_streams() {
    let s1 = vec![
        KeyValue { key: b"a".to_vec(), value: Some(b"1".to_vec()) },
        KeyValue { key: b"d".to_vec(), value: Some(b"4".to_vec()) },
    ];
    let s2 = vec![
        KeyValue { key: b"b".to_vec(), value: Some(b"2".to_vec()) },
        KeyValue { key: b"e".to_vec(), value: Some(b"5".to_vec()) },
    ];
    let s3 = vec![
        KeyValue { key: b"c".to_vec(), value: Some(b"3".to_vec()) },
    ];

    let merged = MergeIterator::new(vec![s1.into_iter(), s2.into_iter(), s3.into_iter()]);
    let keys: Vec<Vec<u8>> = merged.map(|kv| kv.key).collect();

    assert_eq!(keys, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec(), b"d".to_vec(), b"e".to_vec()]);
}
