use std::sync::Arc;

use log::{debug, error, info, trace};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    task::JoinHandle,
};
use tokio_context::context::{Context, RefContext};

use crate::protocol::{Segment, CURRENT_VERSION};
use crate::{
    command::Command,
    config::Config,
    connection::connection::Connection,
    protocol::{frame::Frame, kind::Kind},
};

pub fn start_cmd_server(ctx: RefContext, cfg: Arc<Config>) -> anyhow::Result<JoinHandle<()>> {
    let addr = String::from(&cfg.listen_addr);
    let port = cfg.listen_port;
    let bind = format!("{}:{}", addr, port);
    info!("Command server listening at: {}", bind);
    // info!("Ready to accept command incoming!");

    let handler = tokio::spawn(async move {
        let (mut done_ctx, _handler) = Context::with_parent(&ctx, None);
        let tcp_listener = TcpListener::bind(bind).await.unwrap();
        info!("Command server thread startup");
        loop {
            // debug!("Ready to acceot new connection");
            select! {
                _ = done_ctx.done() => {
                    info!("Command server loop stop");
                    break;
                },
                _ = accept(ctx.clone(), cfg.clone(), &tcp_listener) => {
                }
            }
        }
    });
    Ok(handler)
}

async fn accept(ctx: RefContext, cfg: Arc<Config>, tcp_listener: &TcpListener) {
    match tcp_listener.accept().await {
        Ok((socket, addr)) => {
            let addr = addr.to_string();
            info!("Accepet new conn {}", &addr);
            let cfg = cfg.clone();
            let ctx = ctx.clone();
            // 在另外的线程进行处理
            tokio::spawn(async move {
                info!("new connect {}", &addr);
                connection(ctx, cfg, socket).await;
                info!("disconnect {}", &addr);
            });
        }
        Err(err) => {
            error!("Failed accept data, {:?}", err);
        }
    };
}

async fn connection(ctx: RefContext, cfg: Arc<Config>, stream: TcpStream) {
    let (mut ctx, _handler) = Context::with_parent(&ctx, None);
    let mut conn = Connection::new(stream);
    loop {
        select! {
            _ = ctx.done() => {
                break;
            },
            cmd_result = read_cmd(&mut conn) => {
                match cmd_result {
                    Ok(cmd_opt) => {
                        match cmd_opt {
                            Some(cmd) => {
                                debug!("receive new command: {}", cmd);
                                // TODO: execute Command
                            }
                            None => {
                                // remote close connection
                                break;
                            }
                        }

                    }
                    Err(err) => {
                        error!("read data error {:?}", err);
                        break;
                    }
                }
            }
        }
    }
}

async fn read_cmd(conn: &mut Connection) -> anyhow::Result<Option<Command>> {
    let mut frames: Vec<Frame> = Vec::with_capacity(1);
    if !conn.readable().await? {
        return Ok(None);
    }
    loop {
        match conn.read_frame().await? {
            Some(frame) => {
                // 检查帧的合法信
                if frame.header.version.to_byte() != CURRENT_VERSION {
                    return Err(anyhow::anyhow!("incorrect protocol version"));
                }
                if frame.header.kind != Kind::CMD {
                    return Err(anyhow::anyhow!("not command frame"));
                }
                if frame.is_last() {
                    frames.push(frame);
                    return Ok(Some(parse_cmd(&frames)));
                }
                frames.push(frame);
            }
            None => {
                break;
            }
        }
    }
    Ok(None)
}

fn parse_cmd(frames: &[Frame]) -> Command {
    let capacity: usize = frames.iter().map(|i| i.length.inner_value() as usize).sum();
    let mut payload = Vec::with_capacity(capacity as usize);
    // concat payload
    for frame in frames {
        payload.extend_from_slice(&frame.payload);
    }
    trace!(
        "before decode payload, frames_len={}, capacity={}, payload={:?}",
        frames.len(),
        capacity,
        &payload
    );
    let payload = &payload[..];
    payload.into()
}

#[cfg(test)]
mod test {}
