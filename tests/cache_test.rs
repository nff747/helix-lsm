use helix_lsm::block_cache::BlockCache;

#[test]
fn test_block_cache_lru_eviction() {
    let cache = BlockCache::new(2);
    cache.insert(1, b"block_1".to_vec());
    cache.insert(2, b"block_2".to_vec());

    assert!(cache.get(1).is_some());
    // Insert 3 should evict 2 (since 1 was accessed)
    cache.insert(3, b"block_3".to_vec());

    assert!(cache.get(1).is_some());
    assert!(cache.get(2).is_none());
    assert!(cache.get(3).is_some());
}
