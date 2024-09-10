use std::sync::Arc;

use log::{debug, trace};

use crate::{
    command::Command,
    connection::manager::ConnectionManager,
    protocol::{frame::Frame, length::Length},
};

pub async fn to_cluster(conn_manager: &ConnectionManager, command: &Command) -> anyhow::Result<()> {
    encode_to_frame_and_send(conn_manager, command).await?;
    Ok(())
}

async fn encode_to_frame_and_send(
    conn_manager: &ConnectionManager,
    command: &Command,
) -> anyhow::Result<()> {
    trace!("cluster#encode_to_frame_and_send {}", command);
    let payload = command.encode()?;
    let mut chunks = payload.chunks(256).peekable();
    let chunks_size = chunks.len();
    let mut current_chunk = 0;
    loop {
        if chunks.peek().is_none() {
            trace!("command chunk is none, break loop");
            break;
        }
        if let Some(chunk) = chunks.next() {
            current_chunk += 1;
            trace!("next chunk curreny chunk {}", current_chunk);
            let is_last = current_chunk == chunks_size;
            let frame_head;
            if is_last {
                frame_head = crate::protocol::head::Head::FIN;
            } else {
                frame_head = crate::protocol::head::Head::UNFIN;
            }
            let mut frame = Frame::new();
            let mut payload = Vec::with_capacity(chunk.len());
            payload.extend_from_slice(chunk);
            frame
                .set_head(frame_head)
                .set_length(Length::new(chunk.len() as u8))
                .set_payload(payload);
            // TODO: next step save to raft log and broadcast frame to other node
            debug!("save frame to raft log and broadcast frame to other node");
            let all_conn = conn_manager.all_conn().await?;
            trace!("other node all conn count {}", all_conn.len());
            for conn in all_conn {
                if conn.node.is_self {
                    continue;
                }
                if let Err(err) = conn.write_frame(&mut frame).await {
                    log::error!(
                        "send frame to node {} throws error case: {:?}",
                        &conn.node.get_connection_endpoint(),
                        err
                    );
                }
            }
        }
    }

    Ok(())
}
