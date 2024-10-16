use crate::postman::PostMessage;
use crate::runtime::Runtime;
use crate::{
    config::Config,
    node::{Node, NodeManager, NodeMsg, ShareNodeTable},
};
use log::{error, info, trace};
use socket2::{Domain, Protocol, Socket, Type};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ptr::hash;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::mpsc::Receiver;
use tokio::{net::UdpSocket, select, sync::mpsc, task::JoinHandle, time::interval};
use tokio_context::context::{Context, RefContext};
use uuid::Uuid;

pub struct Discover {
    cfg: Arc<Config>,
    started: bool,
    pub node_id: u64,
}

impl Discover {
    pub fn new(cfg: Arc<Config>) -> Discover {
        let node_id = Uuid::new_v4().to_string();
        let mut hasher = DefaultHasher::new();
        node_id.hash(&mut hasher);
        let node_id = hasher.finish();
        Discover {
            cfg,
            started: false,
            node_id,
        }
    }

    pub fn start(&mut self, app: Arc<Runtime>, parent_ctx: RefContext) -> anyhow::Result<()> {
        let cfg = self.cfg.clone();
        if self.started {
            // 已经启动
            return Ok(());
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
        let interval_duration = cfg.disc_multicast_interval.clone();
        let ctx = parent_ctx.clone();
        let online_node_msg = NodeMsg::new(self.node_id, "", self.cfg.listen_port, true);
        let online_node_msg = serde_json::to_string(&online_node_msg)?;
        let offline_node_msg = NodeMsg::new(self.node_id, "", self.cfg.listen_port, false);
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
                        multicast_to_other_node(&socket_ref, multicast_addr.clone(), offline_message).await;
                        break;
                    }
                    _ = timeout_interval.tick() => {
                        multicast_to_other_node(&socket_ref, multicast_addr.clone(), online_message).await;
                    }
                }
            }
        });

        // 接受组播消息的任务
        let socket_ref = socket.clone();
        let my_id_copy = self.node_id.clone();
        info!("This Node ID: {}", my_id_copy);

        let ctx = parent_ctx.clone();
        let app_copy = app.clone();
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
                    Some(node) = recv_from_other_node(socket_ref.clone(), my_id_copy, &mut buf) => {
                        if !node.is_self {
                            trace!("From other node {:?}", &node);
                        }
                        if let Err(err) = app_copy.postman.send(Box::new(node)).await {
                            error!("send node message error {:?}", err);
                        }
                    }
                }
            }
        });
        self.started = true;
        Ok(())
    }
}

async fn multicast_to_other_node(
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
    my_id: u64,
    buf: &mut [u8],
) -> Option<Node> {
    let (len, addr) = socket_ref.recv_from(buf).await.expect("Failed to receive");
    let msg = String::from_utf8_lossy(&buf[..len]);
    let msg = msg.to_string();
    trace!("recv node ping msg {}", &msg);
    match serde_json::from_str::<NodeMsg>(&msg) {
        Ok(msg) => {
            let is_self = msg.id == my_id;
            Some(Node::new(
                &addr.ip().to_string(),
                msg.id,
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
    app: Arc<Runtime>,
    ctx: RefContext,
    cfg: Arc<Config>,
    node_manager: ShareNodeTable,
    mut recv: Receiver<Box<dyn PostMessage>>,
) -> anyhow::Result<JoinHandle<()>> {
    let mut discover = Discover::new(cfg.clone());
    discover.start(app.clone(), ctx.clone())?;

    let cfg_copy = cfg.clone();
    let ctx_copy = ctx.clone();
    let app_copy = app.clone();
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
                _ = on_ping_node(app_copy.as_ref(), &mut recv, &mut node_manager_copy) => {},
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
    app: &Runtime,
    recv: &mut Receiver<Box<dyn PostMessage>>,
    node_manager: &mut ShareNodeTable,
) -> anyhow::Result<()> {
    if let Some(msg) = recv.recv().await {
        if let Some(node) = msg.as_any().downcast_ref::<Node>() {
            trace!("Recv node ping {:?}", node);
            node_manager.ping(node.clone()).await?;
        }
    }
    Ok(())
}
