use crate::cmd_server::connection;
use crate::command::hello::HelloCmd;
use crate::command::Command;
use crate::protocol::frame::Frame;
use log::debug;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::net::TcpSocket;
use tokio_context::context::RefContext;

mod cluster;
mod cmd_server;
mod command;
mod config;
mod connection;
mod db;
mod discover;
mod node;
mod postman;
mod protocol;
mod runtime;
mod until;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let addr = "127.0.0.1:7111".parse().unwrap();
    let socket = TcpSocket::new_v4()?;
    let stream = socket.connect(addr).await?;
    let (ctx, ctx_handler) = RefContext::new();
    let (tx, rx) = tokio::sync::mpsc::channel(10);
    let conn_handler = tokio::spawn(async move {
        connection(None, ctx, stream, Some(rx)).await;
    });
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 1. hello_cmd_test
    // hello_cmd_test(&tx, 10).await?;
    // 2. ping test
    ping_test(&tx, 10, Duration::from_secs(1)).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;
    ctx_handler.cancel();
    conn_handler.await?;
    Ok(())
}

async fn hello_cmd_test(tx: &Sender<Vec<Frame>>, times: usize) -> anyhow::Result<()> {
    let mut valid = false;
    for _ in 0..times {
        let send_command = Command::new(Box::new(HelloCmd { valid }), None);
        valid = !valid;
        debug!("编码Command为数据帧");
        let frames = send_command.encode_to_frames()?;
        debug!("帧数量: {}", frames.len());
        tx.send(frames).await?;
    }
    Ok(())
}

async fn ping_test(
    tx: &Sender<Vec<Frame>>,
    times: usize,
    interval: Duration,
) -> anyhow::Result<()> {
    for _ in 0..times {
        debug!("帧数量: 1");
        let ping = Frame::new_ping();
        tx.send(vec![ping]).await?;
        tokio::time::sleep(interval).await;
    }
    Ok(())
}
