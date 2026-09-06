use std::collections::BTreeMap;
use crc32fast::Hasher;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: String,
    pub address: String,
}

/// Consistent Hashing Ring with Virtual Nodes for distributed partition routing.
pub struct ConsistentHashRing {
    virtual_nodes: usize,
    ring: BTreeMap<u32, Node>,
}

impl ConsistentHashRing {
    pub fn new(virtual_nodes: usize) -> Self {
        Self {
            virtual_nodes,
            ring: BTreeMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        for v in 0..self.virtual_nodes {
            let vnode_key = format!("{}-vnode-{}", node.id, v);
            let mut hasher = Hasher::new();
            hasher.update(vnode_key.as_bytes());
            let hash = hasher.finalize();
            self.ring.insert(hash, node.clone());
        }
    }

    pub fn get_node(&self, key: &[u8]) -> Option<&Node> {
        if self.ring.is_empty() {
            return None;
        }

        let mut hasher = Hasher::new();
        hasher.update(key);
        let hash = hasher.finalize();

        // Find the first node >= hash, or wrap around to the first node
        match self.ring.range(hash..).next() {
            Some((_, node)) => Some(node),
            None => self.ring.values().next(),
        }
    }
}
