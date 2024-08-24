use std::{io::Cursor, sync::Arc};

use bytes::{Buf, BytesMut};
use log::{debug, error, info, trace};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    select,
};
use tokio_context::context::{Context, RefContext};

use crate::{
    command::Command,
    config::Config,
    protocol::{Frame, FrameMatchResult, Operator},
};

pub struct Connection {
    stream: TcpStream,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            buffer: BytesMut::new(),
        }
    }

    /// Read a frame from the connection.
    ///
    /// Returns `None` if EOF is reached
    pub async fn read_frame(&mut self) -> anyhow::Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // The remote closed the connection. For this to be
                // a clean shutdown, there should be no data in the
                // read buffer. If there is, this means that the
                // peer closed the socket while sending a frame.
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(anyhow::anyhow!("connection reset by peer"));
                }
            }
        }
    }

    pub fn parse_frame(&mut self) -> anyhow::Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        trace!("before check buff: {:?}", &self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(state) => match state {
                FrameMatchResult::Complete => {
                    trace!("Complete, matched frame success");
                    buf.set_position(0);
                    let frame = Frame::parse(&mut buf)?;
                    let len = buf.position() as usize;
                    self.buffer.advance(len);
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
    pub async fn write_frame(&mut self, frame: &Frame) -> anyhow::Result<()> {
        // implementation here
        todo!()
    }
}

pub async fn start_cmd_server(ctx: RefContext, cfg: Arc<Config>) -> anyhow::Result<()> {
    let addr = String::from(&cfg.listen_addr);
    let port = cfg.listen_port;
    let bind = format!("{}:{}", addr, port);
    info!("Listening at: {}", bind);
    let tcp_listener = TcpListener::bind(bind).await?;
    let (mut done_ctx, _handler) = Context::with_parent(&ctx, None);
    // info!("Ready to accept command incoming!");
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
    Ok(())
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
                debug!("new connect {}", &addr);
                connection(ctx, cfg, socket).await;
                debug!("disconnect {}", &addr);
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
                                debug!("Recv Command: {:?}", cmd);
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
    let mut op: Operator = Operator::UNKNOWN;
    loop {
        match conn.read_frame().await? {
            Some(frame) => {
                if op == Operator::UNKNOWN {
                    op = frame.op.clone();
                }
                if frame.is_last() {
                    frames.push(frame);
                    return Ok(Some(parse_cmd(&frames)));
                }
                if frame.op != op {
                    // different kind frame
                    return Err(anyhow::anyhow!("different kind frame"));
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
    payload.into()
}

#[cfg(test)]
mod test {}
