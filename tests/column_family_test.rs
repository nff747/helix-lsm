use helix_lsm::column_family::ColumnFamilyRegistry;

#[test]
fn test_column_family_resolution() {
    let mut reg = ColumnFamilyRegistry::new();
    assert_eq!(reg.get_id("default"), Some(0));

    let analytics_id = reg.register("analytics");
    assert_eq!(analytics_id, 1);
    assert_eq!(reg.get_id("analytics"), Some(1));
    assert_eq!(reg.get_id("missing"), None);
}
