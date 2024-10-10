use log::{debug, error};
use tokio::{
    io::AsyncWriteExt,
    net::TcpSocket,
};
use tokio::io::BufWriter;

use crate::command::Command;
use crate::command::hello::HelloCmd;

mod cluster;
mod cmd_server;
mod command;
mod config;
mod connection;
mod db;
mod discover;
mod node;
mod protocol;
mod runtime;
mod until;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let addr = "127.0.0.1:7111".parse().unwrap();
    let socket = TcpSocket::new_v4()?;
    let stream = socket.connect(addr).await?;
    let mut write_stream = BufWriter::new(stream);
    let mut valid = false;
    for _ in 0..10000 {
        if let Err(_) = write_stream.get_ref().writable().await {
            error!("连接断开");
            break;
        }
        let send_command = Command::new(Box::new(HelloCmd {
            valid
        }), None);
        valid = !valid;
        debug!("编码Command为数据帧");
        let frames = send_command.encode_to_frames()?;
        debug!("帧数量: {}", frames.len());
        let mut send_frame_cnt = 0;
        for mut frame in frames {
            debug!("发送帧: seq={}, data={:?}", send_frame_cnt, frame.encode());
            write_stream.write(frame.encode()).await?;
            send_frame_cnt += 1;
        }
        write_stream.flush().await?;
        debug!("发送完成, 帧数={}", send_frame_cnt);
    }
    Ok(())
}
