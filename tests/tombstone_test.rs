use helix_lsm::tombstone_gc::TombstoneGc;
use helix_lsm::merge_iter::KeyValue;

#[test]
fn test_tombstone_purged_at_bottom_level() {
    let entries = vec![
        KeyValue { key: b"live_key".to_vec(), value: Some(b"val".to_vec()) },
        KeyValue { key: b"dead_key".to_vec(), value: None },
    ];

    let preserved = TombstoneGc::filter_bottom_level(entries.clone(), false);
    assert_eq!(preserved.len(), 2);

    let purged = TombstoneGc::filter_bottom_level(entries, true);
    assert_eq!(purged.len(), 1);
    assert_eq!(purged[0].key, b"live_key");
}
