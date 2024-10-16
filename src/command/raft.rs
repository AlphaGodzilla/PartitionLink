use crate::command::{CommandType, ExecutableCommand};
use crate::db::database::Database;
use crate::db::dbvalue::DBValue;
use async_trait::async_trait;
use std::any::Any;
use std::fmt::{Display, Formatter};


use crate::proto::command_message::Cmd;
use crate::proto::RaftCmd;
use crate::runtime::Runtime;
use protobuf::Message as PbMessage;
use raft::prelude::Message;
// #[derive(Clone)]
// pub struct RaftCmd {
//     pub body: DBValue,
// }

#[async_trait]
impl ExecutableCommand for RaftCmd {
    fn cmd_type(&self) -> CommandType {
        CommandType::WRITE
    }

    async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>> {
        Ok(None)
    }

    fn to_cmd(&self) -> anyhow::Result<Cmd> {
        Ok(Cmd::Raft(self.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Display for RaftCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RaftCmd")
    }
}

impl TryFrom<Cmd> for RaftCmd {
    type Error = anyhow::Error;

    fn try_from(value: Cmd) -> Result<Self, Self::Error> {
        if let Cmd::Raft(cmd) = value {
            Ok(cmd)
        }else {
            Err(anyhow::anyhow!("invalid command"))
        }
    }
}

impl RaftCmd {
    pub fn to_raft_message(&self) -> anyhow::Result<raft::prelude::Message> {
        Ok(Message::parse_from_bytes(&self.body[..])?)
    }
}
