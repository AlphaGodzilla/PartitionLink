use log::{debug, trace};

use crate::{command::Command, connection::manager::ConnectionManager, protocol::frame::Frame};

pub mod raft;

pub async fn broadcast(conn_manager: &ConnectionManager, command: &Command) -> anyhow::Result<()> {
    let mut frames = command.encode_to_frames()?;
    send_frame(conn_manager, &mut frames[..]).await?;
    Ok(())
}

async fn send_frame(conn_manager: &ConnectionManager, frames: &mut [Frame]) -> anyhow::Result<()> {
    debug!("保存frame到raft日志，并且广播给其它节点");
    let all_conn = conn_manager.all_conn().await?;
    trace!("其它节点数量 {}", all_conn.len());
    for conn in all_conn {
        if conn.node.is_self {
            continue;
        }
        if let Err(err) = conn.write_frame(frames).await {
            log::error!(
                "send frame to node {} throws error case: {:?}",
                &conn.node.get_connection_endpoint(),
                err
            );
        }
    }
    Ok(())
}
