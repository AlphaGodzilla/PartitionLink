use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_context::context::RefContext;

use crate::cmd_server::start_cmd_server;
use crate::command::Command;
use crate::config::Config;
use crate::connection::manager::ConnectionManager;
use crate::db::database::{start_db_cmd_channel, Database};
use crate::discover::start_discover;
use crate::node::{NodeTable, ShareNodeTable};

// 运行时作用：负责节点发现和数据同步
pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Runtime {}
    }

    pub fn start(
        &self,
        ctx: &RefContext,
        cfg: Arc<Config>,
        db: Database,
        db_recv: mpsc::Receiver<Command>,
    ) -> anyhow::Result<(JoinHandle<()>, JoinHandle<()>, JoinHandle<()>)> {
        // 初始化节点表
        let node_table = NodeTable::new(cfg.clone());
        let node_manager = ShareNodeTable::new(node_table);

        // 初始化连接管理器
        let conn_manager = ConnectionManager::new(node_manager.clone());

        // 启动节点发现
        let discover_handler = start_discover(ctx, cfg.clone(), node_manager.clone())?;

        // 启动cmd server用于监听其它进程发送过来的命令
        let cmd_server_handler = start_cmd_server(ctx.clone(), cfg.clone())?;

        // 启动db_cmd_channel, 用于处理来自本地或者cmd_server的db命令
        let db_cmd_channel = start_db_cmd_channel(ctx.clone(), db, db_recv, conn_manager.clone())?;
        Ok((discover_handler, cmd_server_handler, db_cmd_channel))
    }
}
