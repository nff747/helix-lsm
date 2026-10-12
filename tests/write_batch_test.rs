use helix_lsm::write_batch::{BatchOp, WriteBatch};

#[test]
fn test_write_batch_operations() {
    let mut batch = WriteBatch::new();
    batch.put(b"k1", b"v1");
    batch.put(b"k2", b"v2");
    batch.delete(b"k1");

    assert_eq!(batch.count(), 3);
    assert_eq!(batch.operations[2], BatchOp::Delete(b"k1".to_vec()));

    batch.clear();
    assert_eq!(batch.count(), 0);
}
