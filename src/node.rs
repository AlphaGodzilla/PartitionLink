use crate::config::Config;
use ahash::{HashMap, HashMapExt};
use async_trait::async_trait;
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::until::now_ts;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeMsg {
    pub id: String,
    pub addr: String,
    pub port: usize,
    pub online: bool,
}

impl NodeMsg {
    pub fn new(id: &str, addr: &str, port: usize, online: bool) -> Self {
        NodeMsg {
            id: String::from(id),
            addr: String::from(addr),
            port,
            online,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct Node {
    pub addr: String,
    pub id: String,
    pub port: usize,
    pub is_self: bool,
    pub online: bool,
}

impl Node {
    pub fn new(addr: &str, id: &str, port: usize, is_self: bool, online: bool) -> Self {
        Node {
            id: String::from(id),
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

#[async_trait]
pub trait NodeManager {
    // 刷新Node
    async fn ping(&mut self, node: Node) -> anyhow::Result<()>;
    // 检查Node是否存在
    async fn exist(&self, id: &str) -> anyhow::Result<bool>;
    // 清理Node列表
    async fn prune(&mut self) -> anyhow::Result<usize>;
    // 返回其它节点列表
    async fn get_other_nodes(&self) -> Vec<Arc<Node>>;
}

#[derive(Clone)]
pub struct NodeTable {
    cfg: Arc<Config>,
    nodes: HashMap<String, Arc<Node>>,
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
}

#[async_trait]
impl NodeManager for NodeTable {
    async fn ping(&mut self, node: Node) -> anyhow::Result<()> {
        let id = String::from(&node.id);
        if !node.online {
            // 节点下线，从节点表中删除节点
            self.nodes.remove(&id);
            self.expire_until.remove(&id);
            info!("Node offline remove {}", serde_json::to_string(&node)?);
            return Ok(());
        }
        let id_copy = id.clone();
        if !self.nodes.contains_key(&id) {
            info!("New Node {}", serde_json::to_string(&node)?);
            self.nodes.insert(id, Arc::new(node));
        }
        let until = now_ts()? + self.cfg.disc_multicast_ttl.as_millis();
        self.expire_until.insert(id_copy, until);
        Ok(())
    }

    async fn exist(&self, id: &str) -> anyhow::Result<bool> {
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
}

#[derive(Clone)]
pub struct ShareNodeTable {
    inner: Arc<Mutex<NodeTable>>,
}

impl ShareNodeTable {
    pub fn new(node_table: NodeTable) -> Self {
        ShareNodeTable {
            inner: Arc::new(Mutex::new(node_table)),
        }
    }
}

#[async_trait]
impl NodeManager for ShareNodeTable {
    // 刷新Node
    async fn ping(&mut self, node: Node) -> anyhow::Result<()> {
        let mut nt = self.inner.lock().await;
        nt.ping(node).await
    }
    // 检查Node是否存在
    async fn exist(&self, id: &str) -> anyhow::Result<bool> {
        let nt = self.inner.lock().await;
        nt.exist(id).await
    }
    // 清理Node列表
    async fn prune(&mut self) -> anyhow::Result<usize> {
        let mut nt = self.inner.lock().await;
        nt.prune().await
    }
    // 返回其它节点列表
    async fn get_other_nodes(&self) -> Vec<Arc<Node>> {
        let nt = self.inner.lock().await;
        nt.get_other_nodes().await
    }
}
