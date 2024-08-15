use ahash::{HashMap, HashMapExt};
use log::info;

use crate::until::{self, now_ts};

#[derive(Clone, Debug)]
pub struct Node {
    pub addr: String,
    pub id: String,
    pub is_self: bool,
}

impl Node {
    pub fn new(addr: String, id: String, is_self: bool) -> Self {
        Node { addr, id, is_self }
    }
}

#[derive(Clone)]
pub struct NodeTable {
    nodes: HashMap<String, String>,
    expire_until: HashMap<String, u128>,
}

impl NodeTable {
    pub fn new() -> Self {
        NodeTable {
            nodes: HashMap::new(),
            expire_until: HashMap::new(),
        }
    }

    pub fn ping(&mut self, node: Node) -> anyhow::Result<()> {
        let id = String::from(&node.id);
        let addr = String::from(&node.addr);
        self.nodes.insert(id, addr);
        let now = now_ts()?;
        let id = String::from(&node.id);
        self.expire_until.insert(id, now);
        Ok(())
    }

    pub fn exist(&self, id: &str) -> anyhow::Result<bool> {
        let now = until::now_ts()?;
        Ok(self.nodes.contains_key(id) && self.expire_until.get(id).unwrap_or(&0) > &now)
    }

    pub fn prune(&mut self) -> anyhow::Result<usize> {
        let now = until::now_ts()?;
        let mut count = 0;
        for (id, ts) in self.expire_until.iter() {
            if ts < &now {
                self.nodes.remove(id);
                count += 1;
                info!("Node {} disconnect, remove from nodeTable", id)
            }
        }
        self.expire_until.retain(|_key, &mut ts| ts < now);
        Ok(count)
    }
}
