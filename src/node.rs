use std::sync::Arc;
use ahash::{HashMap, HashMapExt};
use log::info;
use crate::config::Config;

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
    cfg: Arc<Config>,
    nodes: HashMap<String, Node>,
    expire_until: HashMap<String, u128>,
}

impl NodeTable {
    pub fn new(cfg: Arc<Config>) -> Self {
        NodeTable {
            cfg,
            nodes: HashMap::new(),
            expire_until: HashMap::new(),
        }
    }

    pub fn ping(&mut self, node: Node) -> anyhow::Result<()> {
        let id = String::from(&node.id);
        let id_copy = id.clone();
        if !self.nodes.contains_key(&id) {
            info!("Find new {:?}", &node);
            self.nodes.insert(id, node);
            info!("Current Nodes: {:?}", self.nodes)
        }
        let until = now_ts()? + self.cfg.disc_multicast_ttl.as_millis();
        self.expire_until.insert(id_copy, until);
        Ok(())
    }

    pub fn exist(&self, id: &str) -> anyhow::Result<bool> {
        let now = now_ts()?;
        Ok(self.nodes.contains_key(id) && self.expire_until.get(id).unwrap_or(&0) > &now)
    }

    pub fn prune(&mut self) -> anyhow::Result<usize> {
        let now = now_ts()?;
        let mut count = 0;
        for (id, ts) in self.expire_until.iter() {
            if ts < &now {
                self.nodes.remove(id);
                count += 1;
                info!("Node {} disconnect, remove", id);
                info!("Current Nodes: {:?}", self.nodes);
            }
        }
        if count > 0 {
            self.expire_until.retain(|_key, &mut ts| ts > now);
        }
        Ok(count)
    }
}
