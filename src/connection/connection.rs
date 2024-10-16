use std::{
    io::{Cursor, Write},
    ops::{Deref, DerefMut},
};

use crate::protocol::frame::FrameMissMatchReason;
use crate::{
    node::Node,
    protocol::frame::{Frame, FrameMatchResult},
};
use bytes::{Buf, BytesMut};
use log::{log_enabled, trace};
use tokio::io::BufWriter;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};

pub struct Connection {
    peer_addr: String,
    read_stream: Mutex<OwnedReadHalf>,
    write_stream: Mutex<BufWriter<OwnedWriteHalf>>,
    read_buf: Mutex<BytesMut>,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let peer_addr = stream.peer_addr().unwrap().to_string();
        let (read, write) = stream.into_split();
        Connection {
            peer_addr,
            read_stream: Mutex::new(read),
            write_stream: Mutex::new(BufWriter::new(write)),
            read_buf: Mutex::new(BytesMut::new()),
        }
    }

    pub fn get_peer_addr(&self) -> &str {
        &self.peer_addr
    }

    pub async fn readable(&self) -> anyhow::Result<bool> {
        Ok(self
            .read_stream
            .lock()
            .await
            .ready(Interest::READABLE)
            .await?
            .is_readable())
    }

    pub async fn writeable(&self) -> anyhow::Result<bool> {
        Ok(self
            .write_stream
            .lock()
            .await
            .get_ref()
            .ready(Interest::WRITABLE)
            .await?
            .is_writable())
    }

    pub async fn is_open(&self) -> bool {
        if let Ok(_) = self.writeable().await {
            return true;
        }
        if let Ok(_) = self.readable().await {
            return true;
        }
        return false;
    }

    /// Read a frame from the connection.
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&self) -> anyhow::Result<Option<Frame>> {
        loop {
            {
                if let Some(frame) = self.parse_frame().await? {
                    return Ok(Some(frame));
                }
            }

            {
                let mut read_stream = self.read_stream.lock().await;
                let mut read_buf = self.read_buf.lock().await;
                if 0 == read_stream.read_buf(read_buf.deref_mut()).await? {
                    if read_buf.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(anyhow::anyhow!("connection reset by peer"));
                    }
                }
                if log_enabled!(log::Level::Trace) {
                    trace!("read_buf {:?}", &read_buf[..])
                }
            }
        }
    }

    pub async fn parse_frame(&self) -> anyhow::Result<Option<Frame>> {
        let mut read_buf = self.read_buf.lock().await;
        let mut buf = Cursor::new(&read_buf[..]);
        trace!("before check buff: {:?}", &read_buf[..]);
        match Frame::check(&mut buf) {
            Ok(state) => match state {
                FrameMatchResult::Complete => {
                    trace!("Complete, matched frame success");
                    buf.set_position(0);
                    let frame = Frame::parse(&mut buf)?;
                    let len = buf.position() as usize;
                    read_buf.advance(len);
                    trace!("got frame {:?}", &frame);
                    Ok(Some(frame))
                }
                FrameMatchResult::Incomplete(reason) => {
                    trace!("Incomplete, reason={}", reason);
                    Ok(None)
                }
                FrameMatchResult::MissMatch(reason) => {
                    trace!("MissMatch: reason={:?}", reason);
                    match reason {
                        FrameMissMatchReason::NoneMagic => {
                            read_buf.clear();
                        }
                        _ => {}
                    }
                    Ok(None)
                }
            },
            Err(err) => Err(err.into()),
        }
    }

    /// Write a frame to the connection.
    pub async fn write_frame(&self, frames: &mut [Frame]) -> anyhow::Result<()> {
        let mut write_stream = self.write_stream.lock().await;
        for frame in frames {
            write_stream.write(frame.encode()).await?;
        }
        write_stream.flush().await?;
        Ok(())
    }
}

pub struct NodeConnection {
    pub node: Node,
    pub conn: Connection,
}

impl NodeConnection {
    pub fn new(node: Node, conn: Connection) -> Self {
        NodeConnection { node, conn }
    }
}

impl Deref for NodeConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl DerefMut for NodeConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

#[cfg(test)]
mod test {
    use bytes::{Buf, BytesMut};

    #[test]
    fn bytes_advance_test() {
        // 创建一个新的 BytesMut 实例，并填充一些数据
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
        let mut buf = BytesMut::from(&data[..]);

        println!("Original buffer: {:?}", buf); // 输出原始缓冲区内容
                                                // 前进两个字节
        buf.advance(2);

        println!("Buffer after advancing 2 bytes: {:?}", buf); // 输出前进后的缓冲区内容
    }

    #[test]
    fn slice_test() {
        let vec = vec![1, 2, 3];
        println!("vec[..]: {:?}", &vec[..]);
        println!("vec[..3]: {:?}", &vec[..3]);
    }
}
