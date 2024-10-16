use std::{net::SocketAddr, sync::Arc};

use ahash::AHashMap;
use log::trace;
use tokio::{net::TcpSocket, sync::Mutex};

use crate::node::{Node, NodeManager, ShareNodeTable};

use super::connection::{Connection, NodeConnection};

#[derive(Clone)]
pub struct ConnectionManager {
    node_table: ShareNodeTable,
    connections: Arc<Mutex<AHashMap<Node, Arc<NodeConnection>>>>,
}

impl ConnectionManager {
    pub fn new(node_table: ShareNodeTable) -> Self {
        ConnectionManager {
            node_table,
            connections: Arc::new(Mutex::new(AHashMap::new())),
        }
    }

    pub fn get_node_manager_ref(&self) -> &ShareNodeTable {
        &self.node_table
    }

    pub async fn all_conn(&self) -> anyhow::Result<Vec<Arc<NodeConnection>>> {
        let nodes = self.node_table.get_other_nodes().await;
        log::trace!("find node table's other node count {}", nodes.len());
        let mut all_conn: Vec<Arc<NodeConnection>> = Vec::with_capacity(nodes.len());
        for node in nodes {
            let conn = self.get(node.as_ref()).await?;
            all_conn.push(conn.clone());
        }
        Ok(all_conn)
    }

    async fn first_conn(&self) -> anyhow::Result<Option<Arc<NodeConnection>>> {
        let nodes = self.node_table.get_other_nodes().await;
        if nodes.len() <= 0 {
            return Ok(None);
        }
        let node = nodes.first().unwrap().as_ref();
        let mut connections = self.connections.lock().await;
        let fetch_conn_result = connections.get(node);
        match fetch_conn_result {
            Some(conn) => {
                if !conn.is_open().await {
                    let conn = new_connection(node).await?;
                    connections.insert(node.clone(), Arc::new(conn));
                    let conn_ref = connections.get(node);
                    return Ok(conn_ref.map(|x| x.clone()));
                } else {
                    return Ok(connections.get(node).map(|x| x.clone()));
                }
            }
            None => {
                let conn = new_connection(node).await?;
                connections.insert(node.clone(), Arc::new(conn));
                Ok(connections.get(node).map(|x| x.clone()))
            }
        }
    }

    pub async fn get(&self, node: &Node) -> anyhow::Result<Arc<NodeConnection>> {
        let mut connections = self.connections.lock().await;
        match connections.get(node) {
            Some(conn) => {
                if conn.is_open().await {
                    return Ok(conn.clone());
                }
                let new_conn = new_connection(node).await?;
                connections.insert(node.clone(), Arc::new(new_conn));
                let conn = connections.get(node).map(|x| x.clone()).unwrap();
                Ok(conn.clone())
            }
            None => {
                let new_conn = new_connection(node).await?;
                connections.insert(node.clone(), Arc::new(new_conn));
                let conn = connections.get(node).map(|x| x.clone()).unwrap();
                Ok(conn.clone())
            }
        }
    }

    pub async fn get_by_id(&self, node_id: &u64) -> anyhow::Result<Option<Arc<NodeConnection>>> {
        if let Some(node) = self.node_table.get_other_node(node_id).await {
            let conn = self.get(node.as_ref()).await?;
            return Ok(Some(conn));
        }
        Ok(None)
    }
}

async fn new_connection(node: &Node) -> anyhow::Result<NodeConnection> {
    let addr: SocketAddr = match node.get_connection_endpoint().parse() {
        Ok(addr) => addr,
        Err(err) => {
            return Err(anyhow::anyhow!("parse connection addr error {:?}", err));
        }
    };
    let socket = TcpSocket::new_v4()?;
    let stream = socket.connect(addr.clone()).await?;
    let conn = Connection::new(stream);
    trace!("new other node connection addr={}, node={:?}", &addr, node);
    Ok(NodeConnection::new(node.clone(), conn))
}
