use std::{
    cell::RefCell,
    error, fmt,
    io::Cursor,
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::RwLock,
};

use async_trait::async_trait;
use bytes::{Buf, BytesMut};
use log::trace;
use r2d2::ManageConnection;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::Mutex,
};

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
                read_stream.readable().await?;

                if 0 == read_stream.read_buf(&mut self.read_buf).await? {
                    // The remote closed the connection. For this to be
                    // a clean shutdown, there should be no data in the
                    // read buffer. If there is, this means that the
                    // peer closed the socket while sending a frame.
                    if self.read_buf.is_empty() {
                        return Ok(None);
                    } else {
                        return Err(anyhow::anyhow!("connection reset by peer"));
                    }
                }
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
                    trace!("MissMatch: reason={}", reason);
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
