use crate::cluster::cluster::start_cluster;
use crate::cmd_server::start_cmd_server;
use crate::config::Config;
use crate::connection::manager::ConnectionManager;
use crate::db::database::{start_db_cmd_channel, Database};
use crate::discover::start_discover;
use crate::node::{NodeTable, ShareNodeTable};
use crate::postman::{Channel, Postman};
use anyhow::anyhow;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_context::context::RefContext;

// 运行时作用：负责节点发现和数据同步
pub struct Runtime {
    // 应用级别的消息邮差
    pub postman: Postman,
    pub cfg: Arc<Config>,
}

impl Runtime {
    pub fn new(cfg: Arc<Config>) -> Self {
        Runtime {
            postman: Postman::new(),
            cfg: cfg.clone(),
        }
    }

    pub fn new_with_default_config() -> Self {
        Self::new(Arc::new(Config::default()))
    }

    pub async fn start(app: Arc<Runtime>, ctx: RefContext) -> anyhow::Result<Vec<JoinHandle<()>>> {
        // 初始化节点表
        let node_table = NodeTable::new(app.cfg.clone());
        let node_manager = ShareNodeTable::new(node_table);

        // 初始化连接管理器
        let conn_manager = ConnectionManager::new(node_manager.clone());

        // 启动节点发现
        let recv = app
            .postman
            .new_channel(crate::postman::Channel::Discover, 16)
            .await;
        if recv.is_none() {
            return Err(anyhow!("Discover通道已被打开，无法启动"));
        }
        let discover_handler = start_discover(
            app.clone(),
            ctx.clone(),
            app.cfg.clone(),
            node_manager.clone(),
            recv.unwrap(),
        )?;

        // 启动cmd server用于监听其它进程发送过来的命令
        let cmd_server_handler = start_cmd_server(app.clone(), ctx.clone(), app.cfg.clone())?;

        // 启动数据库
        let recv = app.postman.new_channel(Channel::DbCmdReq, 32).await;
        if recv.is_none() {
            return Err(anyhow!("数据库通道已被打开，无法启动"));
        }
        let db_recv = recv.unwrap();
        let db = Database::new();
        // 启动db_cmd_channel, 用于处理来自本地或者cmd_server的db命令
        let db_cmd_channel_handler = start_db_cmd_channel(app.clone(), ctx.clone(), db, db_recv)?;

        // 启动集群
        let cluster_mailbox = app.postman.new_channel(Channel::RaftMsg, 32).await;
        if cluster_mailbox.is_none() {
            return Err(anyhow!("集群消息通道已被打开，无法启动"));
        }
        let proposal_mailbox = app.postman.new_channel(Channel::RaftProposal, 32).await;
        if proposal_mailbox.is_none() {
            return Err(anyhow!("提案消息通道已被打开，无法启动"));
        }
        let cluster_handler = start_cluster(
            ctx.clone(),
            app.cfg.clone(),
            app.clone(),
            conn_manager.clone(),
            cluster_mailbox.unwrap(),
            proposal_mailbox.unwrap(),
        )?;

        Ok(vec![
            discover_handler,
            cmd_server_handler,
            db_cmd_channel_handler,
            cluster_handler,
        ])
    }
}
