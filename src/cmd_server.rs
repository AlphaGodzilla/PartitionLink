use std::sync::Arc;

use log::{debug, error, info};
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    select,
};
use tokio_context::context::{Context, RefContext};

use crate::config::Config;

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
                debug!("start conn {}", &addr);
                connection(ctx, cfg, socket).await;
                debug!("discon conn {}", &addr);
            });
        }
        Err(err) => {
            error!("Failed accept data, {:?}", err);
        }
    };
}

async fn connection(ctx: RefContext, cfg: Arc<Config>, mut stream: TcpStream) {
    let (mut ctx, _handler) = Context::with_parent(&ctx, None);
    let mut cmd_batch: Vec<u8> = Vec::with_capacity(cfg.cmd_buff_size * 2);
    let mut buff = [0; 512];
    loop {
        select! {
            _ = ctx.done() => {
                break;
            },
            result = read_stream(&mut stream, &mut buff) => {
                match result {
                    Ok(n) => {
                        if n == 0 {
                            break;
                        }
                        if cmd_batch.len() > cfg.cmd_buff_size {
                            break;
                        }
                        if n > 0 {
                            cmd_batch.extend_from_slice(&buff[..n]);
                        }
                        if (cmd_batch.len() <= cfg.cmd_buff_size) {
                            // 解析命令
                            // 执行命令
                            let body = String::from_utf8_lossy(&cmd_batch[..]);
                            info!("Recv Command: {}", body);
                            cmd_batch.clear();
                            cmd_batch.shrink_to_fit();
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

async fn read_stream(stream: &mut TcpStream, buff: &mut [u8]) -> anyhow::Result<usize> {
    stream.readable().await?;
    match stream.read(buff).await {
        Ok(n) => Ok(n),
        Err(err) => {
            error!("Error reading data {:?}", err);
            Ok(0)
        }
    }
}

#[cfg(test)]
mod test {
    use tokio::fs::File;
    use tokio::io::{self, AsyncReadExt};

    use bytes::BytesMut;

    #[tokio::test]
    async fn main() -> io::Result<()> {
        let mut f = File::open("/Users/user/Desktop/763_rev6.json").await?;
        let mut buffer = BytesMut::with_capacity(10);

        assert!(buffer.is_empty());
        assert!(buffer.capacity() >= 10);

        // note that the return value is not needed to access the data
        // that was read as `buffer`'s internal cursor is updated.
        //
        // this might read more than 10 bytes if the capacity of `buffer`
        // is larger than 10.
        let size = f.read_buf(&mut buffer).await?;
        println!("The bytes: {:?}", &buffer[..]);
        println!("The bytes read size: {}", size);
        Ok(())
    }
}
