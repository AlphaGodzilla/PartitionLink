use std::any::Any;
use std::fmt::Display;
use async_trait::async_trait;
use prost::Message;
use crate::runtime::Runtime;
use crate::db::{database::Database, dbvalue::DBValue};

use super::{
    proto::{ProtoCmd, ProtoCommand, ProtoHashMapGetCmd},
    ExecutableCommand,
};

#[derive(Clone)]
pub struct HashMapGetCmd {
    pub key: String,
    pub member_key: String,
}

#[async_trait]
impl ExecutableCommand for HashMapGetCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::READ
    }

    async fn execute(&self, app: Option<&Runtime>, db: Option<&mut Database>) -> anyhow::Result<Option<DBValue>> {
        if let Some(db) = db {
            if let Some(value) = db.get(&self.key) {
                return match value {
                    DBValue::Hash(hash) => {
                        Ok(hash.get(&self.member_key).map(|x| x.clone()))
                    }
                    _ => Ok(None),
                }
            }
        }
        Ok(None)
    }

    fn encode(&self) -> anyhow::Result<bytes::Bytes> {
        let mut buff = bytes::BytesMut::new();
        let msg = ProtoHashMapGetCmd {
            key: self.key.clone(),
            member_key: self.member_key.clone(),
        };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> ProtoCmd {
        ProtoCmd::HashMapGetCmd
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Display for HashMapGetCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashGet {}", &self.key)
    }
}

impl From<ProtoHashMapGetCmd> for HashMapGetCmd {
    fn from(value: ProtoHashMapGetCmd) -> Self {
        HashMapGetCmd {
            key: value.key,
            member_key: value.member_key,
        }
    }
}

impl TryFrom<ProtoCommand> for HashMapGetCmd {
    type Error = anyhow::Error;

    fn try_from(value: ProtoCommand) -> Result<Self, Self::Error> {
        let cmd = ProtoHashMapGetCmd::decode(&value.value[..])?;
        Ok(cmd.into())
    }
}
