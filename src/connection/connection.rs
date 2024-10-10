use std::{
    io::{Cursor, Write},
    ops::{Deref, DerefMut},
};

use bytes::{Buf, BytesMut};
use log::{info, trace};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};

use crate::protocol::frame::FrameMissMatchReason;
use crate::{
    node::Node,
    protocol::frame::{Frame, FrameMatchResult},
};

pub struct Connection {
    read_stream: Mutex<OwnedReadHalf>,
    write_stream: Mutex<OwnedWriteHalf>,
    read_buf: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        let (read, write) = stream.into_split();
        Connection {
            read_stream: Mutex::new(read),
            write_stream: Mutex::new(write),
            read_buf: BytesMut::new(),
        }
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
    pub async fn read_frame(&mut self) -> anyhow::Result<Option<Frame>> {
        loop {
            {
                if let Some(frame) = self.parse_frame()? {
                    return Ok(Some(frame));
                }
            }

            {
                let mut read_stream = self.read_stream.lock().await;
                if 0 == read_stream.read_buf(&mut self.read_buf).await? {
                    if self.read_buf.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(anyhow::anyhow!("connection reset by peer"));
                    }
                }
                info!("read_buf {:?}", &self.read_buf[..])
            }
        }
    }

    pub fn parse_frame(&mut self) -> anyhow::Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.read_buf[..]);
        trace!("before check buff: {:?}", &self.read_buf[..]);
        match Frame::check(&mut buf) {
            Ok(state) => match state {
                FrameMatchResult::Complete => {
                    trace!("Complete, matched frame success");
                    buf.set_position(0);
                    let frame = Frame::parse(&mut buf)?;
                    let len = buf.position() as usize;
                    self.read_buf.advance(len);
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
                            self.read_buf.clear();
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
    pub async fn write_frame(&self, frame: &mut Frame) -> anyhow::Result<()> {
        let mut write_stream = self.write_stream.lock().await;
        write_stream.write_all(frame.encode()).await?;
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
