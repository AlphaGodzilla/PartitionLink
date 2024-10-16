use crate::command::proto::ProtoCmd;
use crate::command::raft::RaftCmd;
use crate::command::{Command, CommandType, ProposalCommand};
use crate::config::Config;
use crate::connection::manager::ConnectionManager;
use crate::db::dbvalue::DBValue;
use crate::db::dbvalue::DBValue::String;
use crate::node::{Node, NodeManager, ProposalAddNode, ShareNodeTable};
use crate::postman::{AsAny, Channel, LetterMessage};
use crate::runtime::Runtime;
use anyhow::anyhow;
use log::{error, info, warn};
use protobuf::Message as PbMessage;
use raft::prelude::{
    ConfChange, ConfChangeType, ConfChangeV2, Entry, EntryType, Message, Snapshot,
};
use raft::storage::MemStorage;
use raft::{RawNode, StateRole};
use std::any::Any;
use std::collections::HashMap;
use std::future::IntoFuture;
use std::ptr::read;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_context::context::{Context, RefContext};

pub struct ClusterNode {
    conn_manager: ConnectionManager,
    raft_group: RawNode<MemStorage>,
    // 来自其它节点的消息
    mailbox: Receiver<Box<dyn LetterMessage>>,
    // 来自本地提案
    proposal_mailbox: Receiver<Box<dyn LetterMessage>>,
}

impl ClusterNode {
    pub fn new(
        cfg: Arc<Config>,
        conn_manager: ConnectionManager,
        mailbox: Receiver<Box<dyn LetterMessage>>,
        proposal_mailbox: Receiver<Box<dyn LetterMessage>>,
    ) -> Self {
        let storage = MemStorage::new();
        ClusterNode {
            conn_manager,
            raft_group: RawNode::with_default_logger(&cfg.raft_config, storage).unwrap(),
            mailbox,
            proposal_mailbox,
        }
    }

    pub fn tick(&mut self) {
        self.raft_group.tick();
    }

    pub fn step(&mut self, msg: Message) -> anyhow::Result<()> {
        Ok(self.raft_group.step(msg)?)
    }

    async fn handle_message(&self, messages: Vec<Message>) {
        if messages.is_empty() {
            return;
        }
        // 将消息按节点分组，
        let mut message_groups = HashMap::new();
        for x in messages {
            message_groups.entry(x.to).or_insert_with(Vec::new).push(x)
        }
        let mut futures = Vec::with_capacity(message_groups.len());
        for (to, messages) in message_groups.drain() {
            let conn_manager_copy = self.conn_manager.clone();
            let node_manager = self.conn_manager.get_node_manager_ref().clone();
            // 对每个node开启并发线程同时发送消息
            let handler = tokio::spawn(async move {
                let node_manager_ref = &node_manager;
                match conn_manager_copy.get_by_id(&to).await {
                    Ok(conn_opt) => {
                        match conn_opt {
                            Some(conn) => {
                                let writeable = conn.writeable().await;
                                if writeable.is_err() || !writeable.unwrap() {
                                    let addr = get_node_addr(node_manager_ref, &to)
                                        .await
                                        .unwrap_or(std::string::String::from(""));
                                    warn!("连接不可写入数据, node_id={}, addr={}", &to, addr);
                                    return;
                                }
                                for message in messages {
                                    // 写入数据
                                    let message_bytes = message.write_to_bytes();
                                    match message_bytes {
                                        Ok(bytes) => {
                                            let command = Command::new(
                                                Box::new(RaftCmd {
                                                    body: DBValue::Bytes(bytes),
                                                }),
                                                None,
                                            );
                                            match command.encode_to_frames() {
                                                Ok(mut frames) => {
                                                    if let Err(err) =
                                                        conn.write_frame(&mut frames[..]).await
                                                    {
                                                        let addr =
                                                            get_node_addr(node_manager_ref, &to)
                                                                .await
                                                                .unwrap_or(
                                                                    std::string::String::from(""),
                                                                );
                                                        error!(
                                                            "帧写入错误, node_id={}, addr={}, {:?}",
                                                            &to, addr, err
                                                        );
                                                    }
                                                }
                                                Err(err) => {
                                                    error!("Command帧编码错误, {:?}", err)
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            error!("Raft消息序列化错误, {:?}", err);
                                        }
                                    }
                                }
                            }
                            None => {
                                warn!(
                                    "尝试建立连接的节点不存在或者为当前节点本身, node_id={}",
                                    &to
                                );
                            }
                        }
                    }
                    Err(err) => {
                        let addr = get_node_addr(node_manager_ref, &to)
                            .await
                            .unwrap_or(std::string::String::from(""));
                        error!(
                            "建立节点连接失败, node_id={}, node_addr={}, detail={:?}",
                            &to, addr, err
                        );
                    }
                }
            });
            futures.push(handler);
        }
        // 等待全部发送线程完成
        for future in futures {
            if let Err(err) = future.await {
                error!("Raft发送线程错误退出, {:?}", err);
            }
        }
    }

    async fn handle_committed_entries(&mut self, committed_entries: Vec<Entry>, app: &Runtime) {
        if committed_entries.is_empty() {
            return;
        }
        for entry in committed_entries {
            if entry.data.is_empty() {
                // From new elected leaders.
                continue;
            }
            let entry_type = entry.get_entry_type();
            match entry_type {
                EntryType::EntryConfChange => {
                    let mut cc = ConfChange::default();
                    cc.merge_from_bytes(&entry.data).unwrap();
                    let cs = self.raft_group.apply_conf_change(&cc).unwrap();
                    self.raft_group.raft.raft_log.store.wl().set_conf_state(cs);
                }
                EntryType::EntryNormal => {
                    let data = entry.get_data();
                    let command: Command = data.into();
                    let cmd = command.inner_ref();
                    if cmd.is_valid() && cmd.is_write_type() && !cmd.is_raft_cmd() {
                        // 写数据库操作命令
                        if let Err(err) = app.postman.send(Box::new(command)).await {
                            error!("发送数据更新命令到本地执行队列错误, {:?}", err);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub async fn poll(&mut self, app: &Runtime) -> anyhow::Result<()> {
        // read mailbox and step raft by message
        loop {
            match self.mailbox.try_recv() {
                Ok(msg) => {
                    if let Some(command) = msg.as_any().downcast_ref::<Command>() {
                        if ProtoCmd::RaftCmd == command.inner_ref().cmd_id() {
                            if let Some(raft_cmd) =
                                command.inner_ref().as_any().downcast_ref::<RaftCmd>()
                            {
                                let raft_message = raft_cmd.to_raft_message()?;
                                self.step(raft_message)?;
                            }
                        }
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow!("cluster mailbox disconnected"))
                }
            }
        }

        // tick
        self.tick();

        // proposal
        loop {
            match self.proposal_mailbox.try_recv() {
                Ok(proposal) => {
                    // 提案增加节点
                    if proposal.as_any().is::<ProposalAddNode>() {
                        if let Some(add_node) = proposal.as_any().downcast_ref::<ProposalAddNode>()
                        {
                            if let Err(err) = self.add_node(add_node) {
                                error!("propose_conf_change error, {:?}", err);
                            }
                        }
                    }
                    // 提案database命令
                    else if proposal.as_any().is::<ProposalCommand>() {
                        if let Some(command) = proposal.as_any().downcast_ref::<ProposalCommand>() {
                            let command = &command.0;
                            if let Err(err) = self.propose_command(command) {
                                error!("propose_command error, {:?}", err);
                            }
                        }
                    }
                    // 其它提案...
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(anyhow!("cluster proposal mailbox disconnected"))
                }
            }
        }

        // check ready
        if !self.raft_group.has_ready() {
            return Ok(());
        }
        let store = self.raft_group.raft.raft_log.store.clone();
        let mut ready = self.raft_group.ready();

        // send out messages
        self.handle_message(ready.take_messages()).await;

        // apply snapshot
        if *ready.snapshot() != Snapshot::default() {
            let s = ready.snapshot().clone();
            if let Err(e) = store.wl().apply_snapshot(s) {
                error!("apply snapshot fail: {:?}, need to retry or panic", e);
                return Ok(());
            }
        }

        // apply commited entry
        self.handle_committed_entries(ready.take_committed_entries(), app)
            .await;
        // persistent raft logs
        if let Err(e) = store.wl().append(ready.entries()) {
            error!("persist raft log fail: {:?}, need to retry or panic", e);
            return Ok(());
        }

        if let Some(hs) = ready.hs() {
            // Raft HardState changed, and we need to persist it.
            store.wl().set_hardstate(hs.clone());
        }

        if !ready.persisted_messages().is_empty() {
            // Send out the persisted messages come from the node.
            self.handle_message(ready.take_persisted_messages()).await;
        }

        // Call `RawNode::advance` interface to update position flags in the raft.
        let mut light_rd = self.raft_group.advance(ready);
        // Update commit index.
        if let Some(commit) = light_rd.commit_index() {
            store.wl().mut_hard_state().set_commit(commit);
        }
        // Send out the messages.
        self.handle_message(light_rd.take_messages()).await;
        // Apply all committed entries.
        self.handle_committed_entries(light_rd.take_committed_entries(), app)
            .await;
        // Advance the apply index.
        self.raft_group.advance_apply();
        Ok(())
    }

    pub fn add_node(&mut self, add_node: &ProposalAddNode) -> anyhow::Result<()> {
        if self.raft_group.raft.state != StateRole::Leader {
            return Ok(());
        }
        let node = &add_node.0;
        let exist = self
            .raft_group
            .raft
            .prs()
            .iter()
            .any(|(id, _)| *id == node.id);
        if exist {
            return Ok(());
        }
        // 发起propose
        let mut cc = ConfChange::default();
        cc.set_node_id(node.id);
        cc.set_change_type(ConfChangeType::AddNode);
        // if let Err(err) = self.raft_group.propose_conf_change(vec![], cc) {
        //     error!("propose_conf_change error, {:?}", err);
        // }
        self.raft_group.propose_conf_change(vec![], cc)?;
        Ok(())
    }

    pub fn propose_command(&mut self, command: &Command) -> anyhow::Result<()> {
        let cmd = command.inner_ref();
        if cmd.is_valid() && cmd.is_write_type() && !cmd.is_raft_cmd() {
            let bytes = command.encode_to_payload()?;
            self.raft_group.propose(vec![], bytes.to_vec())?;
        }
        Ok(())
    }
}

pub async fn get_node_addr(
    node_manager: &(dyn NodeManager + Send + Sync),
    id: &u64,
) -> Option<std::string::String> {
    if let Some(node) = node_manager.get_other_node(id).await {
        return Some(std::string::String::from(&node.addr));
    }
    None
}

pub fn start_cluster(
    ctx: RefContext,
    cfg: Arc<Config>,
    app: Arc<Runtime>,
    conn_manager: ConnectionManager,
    mailbox: Receiver<Box<dyn LetterMessage>>,
    proposal_mailbox: Receiver<Box<dyn LetterMessage>>,
) -> anyhow::Result<JoinHandle<()>> {
    let mut cluster_node = ClusterNode::new(cfg.clone(), conn_manager, mailbox, proposal_mailbox);
    let ctx_copy = ctx.clone();
    let cfg_copy = cfg.clone();
    let handler = tokio::spawn(async move {
        let (mut ctx, _handler) = Context::with_parent(&ctx_copy, None);

        let mut ticker = interval(cfg_copy.raft_loop_interval.clone());
        loop {
            select! {
                _ = ctx.done() => {
                    info!("cluster thread shutdown");
                    break;
                },
                _ = ticker.tick() => {
                    if let Err(err) = cluster_node.poll(&app).await {
                        error!("Raft状态机执行异常, {:?}", err);
                    }
                }
            }
        }
    });
    Ok(handler)
}
