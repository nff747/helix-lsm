use tempfile::tempdir;
use helix_lsm::{HelixDb, EngineOptions};
use helix_lsm::wal::{WalWriter, WalReader};
use helix_lsm::types::{InternalKey, ValueType};
use helix_lsm::distributed::{ConsistentHashRing, Node};
use bytes::Bytes;

#[test]
fn test_basic_crud_operations() {
    let dir = tempdir().unwrap();
    let db = HelixDb::open(dir.path(), EngineOptions::default()).unwrap();

    // 1. Put & Get
    db.put(b"alpha", b"val_alpha").unwrap();
    db.put(b"beta", b"val_beta").unwrap();

    assert_eq!(db.get(b"alpha").unwrap(), Some(Bytes::from("val_alpha")));
    assert_eq!(db.get(b"beta").unwrap(), Some(Bytes::from("val_beta")));
    assert_eq!(db.get(b"non_existent").unwrap(), None);

    // 2. Delete
    db.delete(b"alpha").unwrap();
    assert_eq!(db.get(b"alpha").unwrap(), None);
    assert_eq!(db.get(b"beta").unwrap(), Some(Bytes::from("val_beta")));
}

#[test]
fn test_flush_and_sstable_reads() {
    let dir = tempdir().unwrap();
    let db = HelixDb::open(dir.path(), EngineOptions::default()).unwrap();

    for i in 0..500 {
        let key = format!("key_{:04}", i);
        let val = format!("value_{:04}", i);
        db.put(key.as_bytes(), val.as_bytes()).unwrap();
    }

    // Force flush to L0 SSTable
    db.flush().unwrap();

    // Verify all keys remain accessible from SSTables on disk
    for i in 0..500 {
        let key = format!("key_{:04}", i);
        let val = format!("value_{:04}", i);
        assert_eq!(db.get(key.as_bytes()).unwrap(), Some(Bytes::from(val)));
    }
}

#[test]
fn test_compaction_and_tombstone_purging() {
    let dir = tempdir().unwrap();
    let mut options = EngineOptions::default();
    options.memtable_size_bytes = 1024; // Small to force multiple flushes
    let db = HelixDb::open(dir.path(), options).unwrap();

    // Write batch 1
    for i in 0..200 {
        db.put(format!("k_{:03}", i).as_bytes(), b"initial").unwrap();
    }
    db.flush().unwrap();

    // Update batch 2 (overwrites)
    for i in 0..100 {
        db.put(format!("k_{:03}", i).as_bytes(), b"updated").unwrap();
    }
    db.flush().unwrap();

    // Delete batch 3
    for i in 100..200 {
        db.delete(format!("k_{:03}", i).as_bytes()).unwrap();
    }
    db.flush().unwrap();

    // Run compaction L0 -> L1
    db.compact().unwrap();

    for i in 0..100 {
        assert_eq!(db.get(format!("k_{:03}", i).as_bytes()).unwrap(), Some(Bytes::from("updated")));
    }
    for i in 100..200 {
        assert_eq!(db.get(format!("k_{:03}", i).as_bytes()).unwrap(), None);
    }
}

#[test]
fn test_wal_crash_consistency() {
    let dir = tempdir().unwrap();
    let wal_path = dir.path().join("test.wal");

    {
        let mut writer = WalWriter::create(&wal_path).unwrap();
        let k1 = InternalKey::new(Bytes::from("user1"), 1, ValueType::Value);
        let k2 = InternalKey::new(Bytes::from("user2"), 2, ValueType::Value);
        writer.append(&k1, b"pass1").unwrap();
        writer.append(&k2, b"pass2").unwrap();
        writer.sync().unwrap();
    }

    // Replay WAL
    let mut reader = WalReader::open(&wal_path).unwrap();
    let records = reader.recover().unwrap();

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].key.user_key, Bytes::from("user1"));
    assert_eq!(records[0].value, Bytes::from("pass1"));
    assert_eq!(records[1].key.user_key, Bytes::from("user2"));
    assert_eq!(records[1].value, Bytes::from("pass2"));
}

#[test]
fn test_consistent_hash_ring() {
    let mut ring = ConsistentHashRing::new(32);
    ring.add_node(Node { id: "node-1".into(), address: "10.0.0.1:8000".into() });
    ring.add_node(Node { id: "node-2".into(), address: "10.0.0.2:8000".into() });
    ring.add_node(Node { id: "node-3".into(), address: "10.0.0.3:8000".into() });

    let node_a = ring.get_node(b"partition_key_alpha").unwrap();
    let node_b = ring.get_node(b"partition_key_beta").unwrap();

    assert!(!node_a.id.is_empty());
    assert!(!node_b.id.is_empty());
}

#[test]
fn test_distributed_helix_cluster() {
    use helix_lsm::distributed::DistributedHelixCluster;

    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();
    let db1 = HelixDb::open(dir1.path(), EngineOptions::default()).unwrap();
    let db2 = HelixDb::open(dir2.path(), EngineOptions::default()).unwrap();

    let mut cluster = DistributedHelixCluster::new(64);
    cluster.add_shard(
        Node { id: "node-1".into(), address: "127.0.0.1:9001".into() },
        db1.clone(),
    );
    cluster.add_shard(
        Node { id: "node-2".into(), address: "127.0.0.1:9002".into() },
        db2.clone(),
    );

    assert_eq!(cluster.node_count(), 2);

    // Write keys across the cluster
    for i in 0..100 {
        let key = format!("clustered_key_{}", i);
        let val = format!("clustered_val_{}", i);
        cluster.put(key.as_bytes(), val.as_bytes()).unwrap();
    }

    // Read back all keys through the cluster router
    for i in 0..100 {
        let key = format!("clustered_key_{}", i);
        let expected_val = format!("clustered_val_{}", i);
        let res = cluster.get(key.as_bytes()).unwrap();
        assert_eq!(res, Some(Bytes::from(expected_val)));
    }

    // Delete a key
    cluster.delete(b"clustered_key_42").unwrap();
    assert_eq!(cluster.get(b"clustered_key_42").unwrap(), None);
}

