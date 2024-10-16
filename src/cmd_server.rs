use std::{
    cell::RefCell,
    fmt::{Debug, Error},
    sync::Arc,
};

use bytes::BytesMut;
use log::{debug, error, info, log_enabled, trace, warn};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_context::context::{Context, RefContext};

use crate::command::proto::ProtoCmd;
use crate::protocol::frame;
use crate::protocol::{Segment, CURRENT_VERSION};
use crate::runtime::Runtime;
use crate::{
    command::Command,
    config::Config,
    connection::connection::Connection,
    protocol::{frame::Frame, kind::Kind},
};

pub fn start_cmd_server(
    app: Arc<Runtime>,
    ctx: RefContext,
    cfg: Arc<Config>,
) -> anyhow::Result<JoinHandle<()>> {
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
                _ = accept(app.clone(), ctx.clone(), cfg.clone(), &tcp_listener) => {
                }
            }
        }
    });
    Ok(handler)
}

async fn accept(app: Arc<Runtime>, ctx: RefContext, cfg: Arc<Config>, tcp_listener: &TcpListener) {
    match tcp_listener.accept().await {
        Ok((socket, addr)) => {
            let addr = addr.to_string();
            info!("Accept new conn {}", &addr);
            // let cfg = cfg.clone();
            let ctx = ctx.clone();
            // 在另外的线程进行处理
            tokio::spawn(async move {
                info!("new connect {}", &addr);
                connection(Some(app.as_ref()), ctx, socket, None).await;
                info!("disconnect {}", &addr);
            });
        }
        Err(err) => {
            error!("Failed accept data, {:?}", err);
        }
    };
}

pub async fn connection(
    app: Option<&Runtime>,
    ctx: RefContext,
    stream: TcpStream,
    mut send_msg_box: Option<mpsc::Receiver<Vec<Frame>>>,
) {
    let (mut ctx, _handler) = Context::with_parent(&ctx, None);
    let conn = Connection::new(stream);
    loop {
        if send_msg_box.is_some() {
            select! {
                _ = ctx.done() => {
                    break;
                },
                frames_cnt = send_message(&conn, send_msg_box.as_mut().unwrap()) => {
                    if post_send_message(frames_cnt) {
                        break;
                    }
                },
                message = read_message(&conn) => {
                    if post_read_message(&conn, message, app).await {
                        break;
                    }
                }
            }
        } else {
            select! {
                _ = ctx.done() => {
                    break;
                },
                message = read_message(&conn) => {
                    if post_read_message(&conn, message, app).await {
                        break;
                    }
                }
            }
        }
    }
}

pub enum CmdServerMessage {
    PING,
    PONG,
    CMD(Command),
    ERROR(String),
}

async fn send_message(
    conn: &Connection,
    send_msg_box: &mut mpsc::Receiver<Vec<Frame>>,
) -> anyhow::Result<usize> {
    if let Some(mut frames) = send_msg_box.recv().await {
        trace!("通道中读到帧");
        if !conn.writeable().await? {
            return Ok(0);
        }
        trace!("连接可写");
        conn.write_frame(&mut frames[..]).await?;
        trace!("帧写入连接完成");
        return Ok(frames.len());
    }
    Ok(0)
}

fn post_send_message(result: anyhow::Result<usize>) -> bool {
    match result {
        Ok(size) => {
            trace!("成功发送数据帧数量: {}", size);
            false
        }
        Err(err) => {
            error!("发送数据帧错误: {:?}", err);
            true
        }
    }
}

async fn read_message(conn: &Connection) -> anyhow::Result<Option<CmdServerMessage>> {
    let mut frames: Vec<Frame> = Vec::with_capacity(1);
    if !conn.readable().await? {
        return Ok(None);
    }
    loop {
        match conn.read_frame().await? {
            Some(frame) => {
                // 检查帧合法性
                if frame.header.version.to_byte() != CURRENT_VERSION {
                    return Err(anyhow::anyhow!("incorrect protocol version"));
                }
                // 处理ping帧
                match frame.header.kind {
                    Kind::PING => {
                        return Ok(Some(CmdServerMessage::PING));
                    }
                    Kind::PONG => {
                        return Ok(Some(CmdServerMessage::PONG));
                    }
                    Kind::CMD => {
                        if frame.is_last() {
                            frames.push(frame);
                            return Ok(Some(CmdServerMessage::CMD(parse_cmd(&frames))));
                        }
                        frames.push(frame);
                    }
                    Kind::ERROR => {
                        if frame.is_last() {
                            frames.push(frame);
                            let error_msg = parse_error_message(&frames);
                            return Ok(Some(CmdServerMessage::ERROR(error_msg)));
                        }
                        frames.push(frame);
                    }
                    _ => {
                        warn!(
                            "无法解析数据帧, frome={}, kind={:?}",
                            conn.get_peer_addr(),
                            frame.header.kind
                        );
                    }
                }
            }
            None => {
                break;
            }
        }
    }
    Ok(None)
}

async fn post_read_message(
    conn: &Connection,
    message: anyhow::Result<Option<CmdServerMessage>>,
    app: Option<&Runtime>,
) -> bool {
    match message {
        Ok(message_opt) => {
            match message_opt {
                Some(message) => {
                    // 处理消息
                    if let Err(err) = handle_cmd_server_message(&conn, message, app).await {
                        error!("处理命令错误: {:?}", err);
                        if let Err(reply_err) = try_reply_error(&conn, err).await {
                            error!("回复客户端错误: {:?}", reply_err);
                        }
                    }
                    false
                }
                None => true,
            }
        }
        Err(err) => {
            error!("读取命令错误: {:?}", err);
            // 检查连接状态后返回错误
            if let Err(reply_err) = try_reply_error(&conn, err).await {
                error!("回复客户端错误: {:?}", reply_err);
            }
            true
        }
    }
}

fn parse_cmd(frames: &[Frame]) -> Command {
    let capacity: usize = frames.iter().map(|i| i.length.inner_value() as usize).sum();
    let mut payload = BytesMut::with_capacity(capacity);
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

async fn try_reply_error(conn: &Connection, error: anyhow::Error) -> anyhow::Result<()> {
    if conn.is_open().await {
        if let Ok(w) = conn.writeable().await {
            if w {
                let msg = format!("{}", error);
                let payload = msg.as_bytes();
                let mut frames = frame::build_frames(Kind::ERROR, payload)?;
                conn.write_frame(&mut frames[..]).await?;
            }
        }
    }
    Ok(())
}

fn parse_error_message(frames: &[Frame]) -> String {
    let capacity: usize = frames.iter().map(|i| i.length.inner_value() as usize).sum();
    let mut payload = BytesMut::with_capacity(capacity);
    for frame in frames {
        payload.extend_from_slice(&frame.payload);
    }
    String::from_utf8_lossy(&payload[..]).to_string()
}

async fn handle_cmd_server_message(
    conn: &Connection,
    msg: CmdServerMessage,
    app: Option<&Runtime>,
) -> anyhow::Result<()> {
    // debug!("receive new command: {}", cmd);
    match msg {
        CmdServerMessage::PING => {
            debug!("收到PING帧, from={}", conn.get_peer_addr());
            if conn.writeable().await? {
                // 收到PING帧，回复PONG帧
                let mut pong_frame = vec![Frame::new_pong()];
                conn.write_frame(&mut pong_frame[..]).await?;
            }
        }
        CmdServerMessage::PONG => {
            debug!("收到PONG帧, from={}", conn.get_peer_addr());
        }
        CmdServerMessage::CMD(command) => {
            info!(
                "收到命令: from={}, command={}",
                conn.get_peer_addr(),
                command
            );
            if command.inner_ref().is_raft_cmd() {
                if let Some(app) = app {
                    if let Err(err) = app.postman.send(Box::new(command)).await {
                        error!("发送command到本地raft消息队列错误, {:?}", err);
                    }
                }
            } else {
                // 处理收到的CMD命令
                command.execute(app, None).await?;
            }
        }
        CmdServerMessage::ERROR(err_msg) => {
            warn!(
                "收到错误响应: from={}, error={}",
                conn.get_peer_addr(),
                err_msg
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {}
