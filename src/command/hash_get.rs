use std::fmt::Display;

use prost::Message;

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

impl ExecutableCommand for HashMapGetCmd {
    fn cmd_type(&self) -> super::CommandType {
        super::CommandType::READ
    }

    fn execute(&self, db: &mut Database) -> anyhow::Result<Option<DBValue>> {
        if let Some(value) = db.get(&self.key) {
            match value {
                DBValue::Hash(hash) => {
                    return Ok(hash.get(&self.member_key).map(|x| x.clone()));
                }
                _ => return Ok(None),
            }
        }
        return Ok(None);
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

    fn cmd_id(&self) -> i32 {
        ProtoCmd::HashMapGetCmd as i32
    }
}

impl Display for HashMapGetCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashGet")
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
