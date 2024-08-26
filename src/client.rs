use std::time::Duration;

use log::debug;
use protocol::{frame::Frame, head::Head, length::Length, op::Operator};
use tokio::{
    io::AsyncWriteExt,
    net::TcpSocket,
    time::{self},
};
mod protocol;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let addr = "127.0.0.1:7111".parse().unwrap();
    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;
    // let mut timeout_interval = interval(Duration::from_secs(1));
    let mut frame = Frame::new();
    let payload = String::from("hello");
    let payload = payload.into_bytes();
    frame
        .set_head(Head::FIN)
        .set_op(Operator::OP)
        .set_length(Length::new(payload.len() as u8))
        .set_payload(payload);
    debug!("frame: {:?}", &frame);
    let payload_bytes = frame.encode();
    debug!("frame raw, {:?}", payload_bytes);
    loop {
        time::sleep(Duration::from_secs(1)).await;
        debug!("准备写入, {:?}", payload_bytes);
        stream.write(payload_bytes).await?;
        debug!("写入完成");
    }
    Ok(())
}
