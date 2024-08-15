use std::sync::Arc;

use log::{debug, info};
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_context::context::{Context, RefContext};

use crate::node::{Node, NodeTable};
use crate::{config::Config, discover::Discover};

// 运行时作用：负责节点发现和数据同步
pub struct Runtime {}

impl Runtime {
    pub fn new() -> Self {
        Runtime {}
    }

    pub fn start(&self, ctx: &RefContext, cfg: Arc<Config>) -> anyhow::Result<JoinHandle<()>> {
        let mut node_table = NodeTable::new();
        // 启动discover
        let discover_ctx = ctx.clone();
        let mut discover_rev = start_discover(&discover_ctx, cfg.clone())?;
        // 启动runtime线程
        let cfg_copy = cfg.clone();
        let ctx = ctx.clone();
        let handler = tokio::spawn(async move {
            debug!("Runtime thread startup");
            let (mut ctx, _handler) = Context::with_parent(&ctx, None);
            let mut timeout_interval = interval(cfg_copy.disc_multicast_timout.clone());
            timeout_interval.tick().await;
            loop {
                select! {
                    _ = ctx.done() => {
                        debug!("Runtime thread shutdown");
                        break;
                    },
                    _ = on_ping_node(&mut discover_rev, &mut node_table) => {},
                    _ = timeout_interval.tick() => {
                        if let Ok(prune_cnt) = node_table.prune() {
                            if prune_cnt > 0 {
                                info!("Prune complate, remove node count {}", prune_cnt);
                            }
                        }
                    }
                }
            }
        });
        info!("Startup successful");
        Ok(handler)
    }
}

async fn on_ping_node(
    rev: &mut Option<mpsc::Receiver<Node>>,
    node_table: &mut NodeTable,
) -> anyhow::Result<()> {
    if let Some(recv) = rev {
        if let Some(msg) = recv.recv().await {
            debug!("Rev other node ping {:?}", &msg);
            node_table.ping(msg)?;
        }
    }
    Ok(())
}

fn start_discover(
    ctx: &RefContext,
    cfg: Arc<Config>,
) -> anyhow::Result<Option<tokio::sync::mpsc::Receiver<Node>>> {
    let mut discover = Discover::new(cfg);
    // 启动自动发现
    let rev = discover.start(ctx)?;
    Ok(rev)
}
