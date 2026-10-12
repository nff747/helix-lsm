use helix_lsm::leveled_compaction::LeveledCompactionController;

#[test]
fn test_leveled_thresholds() {
    let controller = LeveledCompactionController::new();
    assert_eq!(controller.max_bytes_for_level(1), 10 * 1024 * 1024);
    assert_eq!(controller.max_bytes_for_level(2), 100 * 1024 * 1024);
    assert_eq!(controller.max_bytes_for_level(3), 1000 * 1024 * 1024);

    assert_eq!(controller.should_compact_l0(3), false);
    assert_eq!(controller.should_compact_l0(4), true);
}
