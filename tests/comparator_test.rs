use helix_lsm::comparator::{BytewiseComparator, KeyComparator};
use std::cmp::Ordering;

#[test]
fn test_bytewise_comparator() {
    let comp = BytewiseComparator;
    assert_eq!(comp.compare(b"abc", b"abc"), Ordering::Equal);
    assert_eq!(comp.compare(b"abc", b"abd"), Ordering::Less);
    assert_eq!(comp.compare(b"z", b"a"), Ordering::Greater);
    assert_eq!(comp.name(), "helix.BytewiseComparator");
}
