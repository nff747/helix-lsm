use helix_lsm::snapshot::{SequenceNumber, Snapshot};

#[test]
fn test_snapshot_visibility() {
    let snap = Snapshot::new(100);
    assert_eq!(snap.is_visible(SequenceNumber(50)), true);
    assert_eq!(snap.is_visible(SequenceNumber(100)), true);
    assert_eq!(snap.is_visible(SequenceNumber(101)), false);
}
