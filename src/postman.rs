use ahash::AHashMap;
use std::any::Any;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, RwLock};

#[derive(Eq, PartialEq, Hash)]
pub enum Channel {
    /// 数据库发送命令
    DbCmdReq,
    /// 集群通信信息
    RaftMsg,
    /// 提案
    RaftProposal,
    /// discover
    Discover,
}

pub trait LetterMessage: Send + Sync + Any {
    /// 消息的频道
    fn channel(&self) -> Channel;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Envelope<T> {
    pub inner: T,
}

pub struct Postman {
    channels: RwLock<AHashMap<Channel, Sender<Box<dyn LetterMessage>>>>,
}

impl Postman {
    pub fn new() -> Self {
        Postman {
            channels: RwLock::new(AHashMap::new()),
        }
    }

    /// 注册通道
    /// 返回一个Option值，None表示channel已注册，Some返回新的消息接收器
    pub async fn new_channel(
        &self,
        channel: Channel,
        buf_size: usize,
    ) -> Option<Receiver<Box<dyn LetterMessage>>> {
        let exist = self.channels.read().await.contains_key(&channel);
        if exist {
            return None;
        }
        let (tx, rx) = mpsc::channel(buf_size);
        let mut channels = self.channels.write().await;
        channels.insert(channel, tx);
        Some(rx)
    }

    /// 发送消息
    pub async fn send(&self, message: Box<dyn LetterMessage>) -> anyhow::Result<bool> {
        if let Some(sender) = self.channels.read().await.get(&message.channel()) {
            sender.send(message).await?;
            return Ok(true);
        }
        Ok(false)
    }
}
