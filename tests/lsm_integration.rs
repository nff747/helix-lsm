use helix_lsm::{LsmEngine, LsmConfig};
use tempfile::tempdir;

#[test]
fn test_lsm_full_lifecycle() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        memtable_threshold_bytes: 128,
        sstable_compaction_threshold: 3,
    };

    let mut engine = LsmEngine::open(dir.path(), config.clone()).unwrap();

    for i in 0..50 {
        let key = format!("user:{:04}", i);
        let val = format!("payload_data_string_{:04}", i);
        engine.put(key.as_bytes(), val.as_bytes()).unwrap();
    }

    for i in 0..50 {
        let key = format!("user:{:04}", i);
        let expected = format!("payload_data_string_{:04}", i);
        let res = engine.get(key.as_bytes()).unwrap().expect("key must exist");
        assert_eq!(res, expected.as_bytes());
    }

    drop(engine);
    let mut reopened = LsmEngine::open(dir.path(), config).unwrap();
    for i in 0..50 {
        let key = format!("user:{:04}", i);
        let expected = format!("payload_data_string_{:04}", i);
        let res = reopened.get(key.as_bytes()).unwrap().expect("key must persist");
        assert_eq!(res, expected.as_bytes());
    }
}
