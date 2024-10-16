use super::ExecutableCommand;
use crate::db::{database::Database, dbvalue::DBValue};
use crate::proto::command_message::Cmd;
use crate::proto::HashGetCmd;
use crate::runtime::Runtime;
use async_trait::async_trait;
use prost::Message;
use std::any::Any;
use std::fmt::Display;

// #[derive(Clone)]
// pub struct HashGetCmd {
//     pub key: String,
//     pub member_key: String,
// }

#[async_trait]
impl ExecutableCommand for HashGetCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::READ
    }

    async fn execute(
        &self,
        app: Option<&Runtime>,
        db: Option<&mut Database>,
    ) -> anyhow::Result<Option<DBValue>> {
        if let Some(db) = db {
            if let Some(value) = db.get(&self.key) {
                return match value {
                    DBValue::Hash(hash) => Ok(hash.get(&self.member_key).map(|x| x.clone())),
                    _ => Ok(None),
                };
            }
        }
        Ok(None)
    }

    fn to_cmd(&self) -> anyhow::Result<Cmd> {
        Ok(Cmd::HashGet(self.clone()))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Display for HashGetCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashGet {}", &self.key)
    }
}

impl TryFrom<Cmd> for HashGetCmd {
    type Error = anyhow::Error;

    fn try_from(value: Cmd) -> Result<Self, Self::Error> {
        if let Cmd::HashGet(hash) = value {
            Ok(hash)
        } else {
            Err(anyhow::Error::msg("Invalid command"))
        }
    }
}
