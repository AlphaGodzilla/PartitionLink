use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use log::{error, info, trace};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{net::UdpSocket, select, sync::mpsc, task::JoinHandle, time::interval};
use tokio_context::context::{Context, RefContext};
use uuid::Uuid;

use crate::{
    config::Config,
    node::{Node, NodeManager, NodeMsg, ShareNodeTable},
};

pub struct Discover {
    cfg: Arc<Config>,
    started: bool,
    pub node_id: String,
}

impl Discover {
    pub fn new(cfg: Arc<Config>) -> Discover {
        let node_id = Uuid::new_v4().to_string();
        Discover {
            cfg,
            started: false,
            node_id: String::from(&node_id),
        }
    }

    pub fn start(
        &mut self,
        parent_ctx: &RefContext,
    ) -> Result<Option<tokio::sync::mpsc::Receiver<Node>>, anyhow::Error> {
        let cfg = self.cfg.clone();
        if self.started {
            // 已经启动
            return Ok(None);
        }
        let multicast_addr = format!("{}:{}", cfg.disc_multicast_group, cfg.disc_multicast_port);
        let multicast_addr = multicast_addr.parse::<SocketAddr>()?;
        let local_addr = format!("0.0.0.0:{}", cfg.disc_multicast_port).parse::<SocketAddr>()?;

        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        socket.set_reuse_port(true)?;
        socket.bind(&local_addr.into())?;
        socket.set_nonblocking(true)?;
        let socket = UdpSocket::from_std(socket.into())?;

        let socket = Arc::new(socket);
        // 加入组播组
        let multicast_ip = if let IpAddr::V4(ipv4) = multicast_addr.ip() {
            ipv4
        } else {
            return Err(anyhow::anyhow!("Multicast IP should be IPv4"));
        };
        // 加入组播
        socket.join_multicast_v4(multicast_ip, Ipv4Addr::UNSPECIFIED)?;

        // 发送组播消息的任务
        let socket_ref = socket.clone();
        let my_id_copy = self.node_id.clone();
        let interval_duration = cfg.disc_multicast_interval.clone();
        let ctx = parent_ctx.clone();
        let online_node_msg = NodeMsg::new(&my_id_copy, "", self.cfg.listen_port, true);
        let online_node_msg = serde_json::to_string(&online_node_msg)?;
        let offline_node_msg = NodeMsg::new(&my_id_copy, "", self.cfg.listen_port, false);
        let offline_node_msg = serde_json::to_string(&offline_node_msg)?;
        tokio::spawn(async move {
            info!("Multicast thread startup {}", &multicast_addr);
            let (mut ctx, _handler) = Context::with_parent(&ctx, None);
            let mut timeout_interval = interval(interval_duration);
            let online_message = online_node_msg.as_bytes();
            let offline_message = offline_node_msg.as_bytes();
            loop {
                tokio::select! {
                    _ = ctx.done() => {
                        info!("Multicast thread shutdown");
                        mutilcast_to_other_node(&socket_ref, multicast_addr.clone(), offline_message).await;
                        break;
                    }
                    _ = timeout_interval.tick() => {
                        mutilcast_to_other_node(&socket_ref, multicast_addr.clone(), online_message).await;
                    }
                }
            }
        });

        // 接受组播消息的任务
        let socket_ref = socket.clone();
        let my_id_copy = self.node_id.clone();
        info!("This Node ID: {}", &my_id_copy);
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let ctx = parent_ctx.clone();
        tokio::spawn(async move {
            info!("Listener thread startup");
            let (mut ctx, _handler) = Context::with_parent(&ctx, None);
            let mut buf = [0; 1024];
            loop {
                tokio::select! {
                    _ = ctx.done() => {
                        info!("Listener thread shutdown");
                        break;
                    },
                    Some(node) = recv_from_other_node(socket_ref.clone(), &my_id_copy, &mut buf) => {
                        if !node.is_self {
                            trace!("From other node {:?}", &node);
                        }
                        if let Err(err) = tx.send(node).await {
                            error!("send node message error {:?}", err);
                        }
                    }
                }
            }
        });

        self.started = true;

        Ok(Some(rx))
    }
}

async fn mutilcast_to_other_node(
    socket_ref: &UdpSocket,
    multicast_addr: SocketAddr,
    message: &[u8],
) {
    socket_ref
        .send_to(message, multicast_addr)
        .await
        .expect("Failed to send");
    trace!("Send ping success");
}

async fn recv_from_other_node(
    socket_ref: Arc<UdpSocket>,
    my_id_copy: &str,
    buf: &mut [u8],
) -> Option<Node> {
    let (len, addr) = socket_ref.recv_from(buf).await.expect("Failed to receive");
    let msg = String::from_utf8_lossy(&buf[..len]);
    let msg = msg.to_string();
    trace!("recv node ping msg {}", &msg);
    match serde_json::from_str::<NodeMsg>(&msg) {
        Ok(msg) => {
            let is_self = msg.id == my_id_copy;
            Some(Node::new(
                &addr.ip().to_string(),
                &msg.id,
                msg.port,
                is_self,
                msg.online,
            ))
        }
        Err(err) => {
            error!("handle node ping message error {:?}", err);
            None
        }
    }
}

pub fn start_discover(
    ctx: &RefContext,
    cfg: Arc<Config>,
    node_manager: ShareNodeTable,
) -> anyhow::Result<JoinHandle<()>> {
    let mut discover = Discover::new(cfg.clone());
    let mut recv = discover.start(ctx)?;

    let cfg_copy = cfg.clone();
    let ctx_copy = ctx.clone();
    let mut node_manager_copy = node_manager.clone();
    let discover_handler = tokio::spawn(async move {
        info!("Discover thread startup");
        let (mut ctx, _handler) = Context::with_parent(&ctx_copy, None);
        let mut timeout_interval = interval(cfg_copy.disc_multicast_ttl_check_interval.clone());
        timeout_interval.tick().await;
        loop {
            select! {
                _ = ctx.done() => {
                    info!("Discover thread shutdown");
                    break;
                },
                _ = on_ping_node(&mut recv, &mut node_manager_copy) => {},
                _ = timeout_interval.tick() => {
                    if let Ok(prune_cnt) = node_manager_copy.prune().await {
                        if prune_cnt > 0 {
                            info!("Prune complete, remove node count {}", prune_cnt);
                        }
                    }
                }
            }
        }
    });
    Ok(discover_handler)
}

async fn on_ping_node(
    rev: &mut Option<mpsc::Receiver<Node>>,
    node_manager: &mut ShareNodeTable,
) -> anyhow::Result<()> {
    if let Some(recv) = rev {
        if let Some(msg) = recv.recv().await {
            trace!("Recv node ping {:?}", &msg);
            node_manager.ping(msg).await?;
        }
    }
    Ok(())
}
