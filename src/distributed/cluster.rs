use std::collections::HashMap;
use std::io;
use std::sync::Arc;
use bytes::Bytes;

use super::ring::{ConsistentHashRing, Node};
use crate::engine::HelixDb;

/// Distributed Helix cluster coordinator.
/// Uses consistent hashing with virtual nodes to deterministically route reads and writes
/// across storage shards.
pub struct DistributedHelixCluster {
    ring: ConsistentHashRing,
    nodes: HashMap<String, Node>,
    shards: HashMap<String, Arc<HelixDb>>,
}

impl DistributedHelixCluster {
    pub fn new(virtual_nodes: usize) -> Self {
        Self {
            ring: ConsistentHashRing::new(virtual_nodes),
            nodes: HashMap::new(),
            shards: HashMap::new(),
        }
    }

    /// Register a cluster node and attach its local HelixDb storage engine shard.
    pub fn add_shard(&mut self, node: Node, db: Arc<HelixDb>) {
        self.ring.add_node(node.clone());
        self.nodes.insert(node.id.clone(), node.clone());
        self.shards.insert(node.id, db);
    }

    /// Resolve the target node and db shard for a given key.
    pub fn route(&self, key: &[u8]) -> Option<(&Node, &Arc<HelixDb>)> {
        let node = self.ring.get_node(key)?;
        let shard = self.shards.get(&node.id)?;
        Some((node, shard))
    }

    /// Routes a write operation to the appropriate shard.
    pub fn put(&self, key: &[u8], value: &[u8]) -> io::Result<u64> {
        match self.route(key) {
            Some((_, shard)) => shard.put(key, value),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No storage nodes available in cluster")),
        }
    }

    /// Routes a read operation to the appropriate shard.
    pub fn get(&self, key: &[u8]) -> io::Result<Option<Bytes>> {
        match self.route(key) {
            Some((_, shard)) => shard.get(key),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No storage nodes available in cluster")),
        }
    }

    /// Routes a delete operation to the appropriate shard.
    pub fn delete(&self, key: &[u8]) -> io::Result<u64> {
        match self.route(key) {
            Some((_, shard)) => shard.delete(key),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "No storage nodes available in cluster")),
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
