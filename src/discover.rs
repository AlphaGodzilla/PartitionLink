use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use log::{debug, error, info};
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{
    net::UdpSocket,
    time::{interval, sleep},
};
use tokio_context::context::{Context, RefContext};
use uuid::Uuid;

use crate::{config::Config, node::Node};

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
        let interval_duration = cfg.disc_multicast_interval_duration.clone();
        let ctx = parent_ctx.clone();
        tokio::spawn(async move {
            debug!("Multicast thread startup {}", &multicast_addr);
            let (mut ctx, _handler) = Context::with_parent(&ctx, None);
            let mut timeout_interval = interval(interval_duration);
            let message = my_id_copy.as_bytes();
            loop {
                tokio::select! {
                    _ = ctx.done() => {
                        debug!("Multicast thread shutdown");
                        break;
                    }
                    _ = timeout_interval.tick() => {
                        mutilcast_to_other_node(&socket_ref, multicast_addr.clone(), message).await;
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
            debug!("Listener thread startup");
            let (mut ctx, _handler) = Context::with_parent(&ctx, None);
            let mut buf = [0; 1024];
            loop {
                tokio::select! {
                    _ = ctx.done() => {
                        debug!("Listener thread shutdown");
                        break;
                    },
                    Some(node) = recv_from_other_node(&socket_ref, &my_id_copy, &mut buf) => {
                        info!("Find other node {:?}", node);
                        // register.notify(node).await
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
    debug!("Send ping success");
}

async fn recv_from_other_node(
    socket_ref: &UdpSocket,
    my_id_copy: &str,
    buf: &mut [u8],
) -> Option<Node> {
    let (len, addr) = socket_ref.recv_from(buf).await.expect("Failed to receive");
    let msg = String::from_utf8_lossy(&buf[..len]);
    debug!("recv node ping msg {}", msg);
    if msg == my_id_copy {
        return None;
    }
    Some(Node::new(addr.to_string(), String::from(my_id_copy), false))
}
