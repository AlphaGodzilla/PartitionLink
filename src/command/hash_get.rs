use std::fmt::Display;

use prost::Message;

use crate::db::{database::Database, dbvalue::DBValue};

use super::ExecutableCommand;

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
        let msg = super::proto::out::HashMapGetCmd {
            key: self.key.clone(),
            member_key: self.member_key.clone(),
        };
        msg.encode(&mut buff)?;
        Ok(buff.freeze())
    }

    fn cmd_id(&self) -> i32 {
        crate::command::proto::out::Cmd::HashMapGetCmd as i32
    }
}

impl Display for HashMapGetCmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HashGet")
    }
}

impl From<super::proto::out::HashMapGetCmd> for HashMapGetCmd {
    fn from(value: super::proto::out::HashMapGetCmd) -> Self {
        HashMapGetCmd {
            key: value.key,
            member_key: value.member_key,
        }
    }
}

impl TryFrom<super::proto::out::Command> for HashMapGetCmd {
    type Error = anyhow::Error;

    fn try_from(value: super::proto::out::Command) -> Result<Self, Self::Error> {
        let cmd = super::proto::out::HashMapGetCmd::decode(&value.value[..])?;
        Ok(cmd.into())
    }
}
