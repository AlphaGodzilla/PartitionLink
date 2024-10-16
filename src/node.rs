use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ptr::hash;
use crate::config::Config;
use ahash::{AHashMap, HashMap, HashMapExt, RandomState};
use async_trait::async_trait;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use crate::postman::{Channel, PostMessage};
use crate::until::now_ts;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeMsg {
    pub id: u64,
    pub addr: String,
    pub port: usize,
    pub online: bool,
}

impl NodeMsg {
    pub fn new(id: u64, addr: &str, port: usize, online: bool) -> Self {
        NodeMsg {
            id,
            addr: String::from(addr),
            port,
            online,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Node {
    pub addr: String,
    pub id: u64,
    pub port: usize,
    pub is_self: bool,
    pub online: bool,
}

impl Node {
    pub fn new(addr: &str, id: u64, port: usize, is_self: bool, online: bool) -> Self {
        Node {
            id,
            addr: String::from(addr),
            port,
            is_self,
            online,
        }
    }

    pub fn get_connection_endpoint(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}

impl PostMessage for Node {
    fn channel(&self) -> Channel {
        Channel::Discover
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
pub struct ProposalAddNode(pub Node);

impl PostMessage for ProposalAddNode {
    fn channel(&self) -> Channel {
        Channel::RaftProposal
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
pub trait NodeManager {
    // 刷新Node
    async fn ping(&mut self, node: Node) -> anyhow::Result<()>;
    // 检查Node是否存在
    async fn exist(&self, id: &u64) -> anyhow::Result<bool>;
    // 清理Node列表
    async fn prune(&mut self) -> anyhow::Result<usize>;
    // 返回其它节点列表
    async fn get_other_nodes(&self) -> Vec<Arc<Node>>;
    // 返回传入节点ID的节点
    async fn get_other_node(&self, id: &u64) -> Option<Arc<Node>>;
}

#[derive(Clone)]
pub struct NodeTable {
    cfg: Arc<Config>,
    nodes: AHashMap<u64, Arc<Node>>,
    expire_until: AHashMap<u64, u128>,
}

impl NodeTable {
    pub fn new(cfg: Arc<Config>) -> Self {
        NodeTable {
            cfg,
            nodes: AHashMap::new(),
            expire_until: AHashMap::new(),
        }
    }
}

#[async_trait]
impl NodeManager for NodeTable {
    async fn ping(&mut self, node: Node) -> anyhow::Result<()> {
        let id = node.id;
        if !node.online {
            // 节点下线，从节点表中删除节点
            self.nodes.remove(&id);
            self.expire_until.remove(&id);
            info!("Node offline remove {}", serde_json::to_string(&node)?);
            return Ok(());
        }
        // let id_copy = id.clone();
        if !self.nodes.contains_key(&id) {
            info!("New Node {}", serde_json::to_string(&node)?);
            self.nodes.insert(id, Arc::new(node));
        }
        let until = now_ts()? + self.cfg.disc_multicast_ttl.as_millis();
        self.expire_until.insert(id, until);
        Ok(())
    }

    async fn exist(&self, id: &u64) -> anyhow::Result<bool> {
        let now = now_ts()?;
        Ok(self.nodes.contains_key(id) && self.expire_until.get(id).unwrap_or(&0) > &now)
    }

    async fn prune(&mut self) -> anyhow::Result<usize> {
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

    async fn get_other_nodes(&self) -> Vec<Arc<Node>> {
        self.nodes
            .values()
            .filter(|x| !x.is_self)
            .map(|x| x.clone())
            .collect()
    }

    async fn get_other_node(&self, id: &u64) -> Option<Arc<Node>> {
        self.nodes.get(id)
            .map(|x| x.clone())
    }
}

#[derive(Clone)]
pub struct ShareNodeTable {
    inner: Arc<RwLock<NodeTable>>,
}

impl ShareNodeTable {
    pub fn new(node_table: NodeTable) -> Self {
        ShareNodeTable {
            inner: Arc::new(RwLock::new(node_table)),
        }
    }
}

#[async_trait]
impl NodeManager for ShareNodeTable {
    // 刷新Node
    async fn ping(&mut self, node: Node) -> anyhow::Result<()> {
        let mut nt = self.inner.write().await;
        nt.ping(node).await
    }
    // 检查Node是否存在
    async fn exist(&self, id: &u64) -> anyhow::Result<bool> {
        let nt = self.inner.read().await;
        nt.exist(id).await
    }
    // 清理Node列表
    async fn prune(&mut self) -> anyhow::Result<usize> {
        let mut nt = self.inner.write().await;
        nt.prune().await
    }
    // 返回其它节点列表
    async fn get_other_nodes(&self) -> Vec<Arc<Node>> {
        let nt = self.inner.read().await;
        nt.get_other_nodes().await
    }

    async fn get_other_node(&self, id: &u64) -> Option<Arc<Node>> {
        let nt = self.inner.read().await;
        nt.get_other_node(id).await
    }
}


#[cfg(test)]
mod test {
    use ahash::RandomState;
    use uuid::Uuid;

    #[test]
    pub fn string_hash_code_test() {
        let hasher = RandomState::new();
        for _ in 0..10 {
            let node_id = Uuid::new_v4().to_string();
            println!("{} -> {}", &node_id, hasher.hash_one(&node_id));
        }
    }
}