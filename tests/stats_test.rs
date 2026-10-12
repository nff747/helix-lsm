use helix_lsm::stats::CompactionStats;

#[test]
fn test_compaction_stats_write_amp() {
    let mut stats = CompactionStats::default();
    stats.record(1000, 2500, 3);
    assert_eq!(stats.num_tables_compacted, 3);
    assert_eq!(stats.write_amplification(), 2.5);
}
