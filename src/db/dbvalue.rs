use std::fmt::Display;

use crate::proto::db_value::Value as DbValueEnum;
use crate::proto::DbValue as PDbValue;
use crate::proto::Hash as PHash;
use crate::proto::List as PList;
use ahash::AHashMap;

#[derive(Clone)]
pub enum DBValue {
    None,
    Boolean(bool),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<DBValue>),
    Hash(AHashMap<String, DBValue>),
}

impl Display for DBValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => {
                write!(f, "DBValue::None")?;
            }
            Self::Boolean(v) => {
                write!(f, "DBValue::{}", &format!("Boolean({})", &v))?;
            }
            Self::String(v) => {
                write!(f, "DBValue::{}", &format!("String({})", &v))?;
            }
            Self::Bytes(v) => {
                write!(f, "DBValue::{}", &format!("Bytes({} bytes)", v.len()))?;
            }
            Self::List(v) => {
                write!(f, "DBValue::List(",)?;
                let mut iter = v.iter();
                if let Some(first) = iter.next() {
                    write!(f, "{}", first)?;
                    for item in iter {
                        write!(f, ",{}", item)?;
                    }
                }
                write!(f, ")")?;
            }
            Self::Hash(h) => {
                write!(f, "DBValue::Hash(\n",)?;
                let mut iter = h.iter();
                if let Some((k, v)) = iter.next() {
                    write!(f, "  {} = {}", k, v)?;
                    for item in iter {
                        write!(f, ",\n{}={}", k, v)?;
                    }
                }
                write!(f, ")")?;
            }
        };
        Ok(())
    }
}

impl From<DBValue> for PDbValue {
    fn from(value: DBValue) -> Self {
        value.to_protobuf()
    }
}

impl DBValue {
    pub fn to_protobuf(&self) -> PDbValue {
        let value = match self {
            // DBValue::None => Some(proto_db_value::Value::NoneDbValue(false)),
            DBValue::None => Some(DbValueEnum::None(false)),
            DBValue::Boolean(v) => Some(DbValueEnum::Bool(v.clone())),
            DBValue::String(v) => Some(DbValueEnum::String(v.clone())),
            DBValue::Bytes(v) => Some(DbValueEnum::Bytes(v.clone())),
            DBValue::List(values) => Some(DbValueEnum::List(PList {
                value: values.iter().map(|x| x.to_protobuf()).collect(),
            })),
            DBValue::Hash(values) => Some(DbValueEnum::Hash(PHash {
                values: values
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_protobuf()))
                    .collect(),
            })),
        };
        PDbValue { value }
    }
}

impl From<PDbValue> for DBValue {
    fn from(value: PDbValue) -> Self {
        if let None = value.value {
            return DBValue::None;
        }
        let value = value.value.unwrap();
        match value {
            DbValueEnum::None(_) => DBValue::None,
            DbValueEnum::Bool(b) => DBValue::Boolean(b),
            DbValueEnum::String(s) => DBValue::String(s),
            DbValueEnum::Bytes(b) => DBValue::Bytes(b),
            DbValueEnum::List(l) => {
                let l: Vec<DBValue> = l.value.into_iter().map(|x| x.into()).collect();
                DBValue::List(l)
            }
            DbValueEnum::Hash(h) => {
                let h: AHashMap<String, DBValue> =
                    h.values.into_iter().map(|(k, v)| (k, v.into())).collect();
                DBValue::Hash(h)
            }
        }
    }
}

impl From<PDbValue> for DbValueEnum {
    fn from(value: PDbValue) -> Self {
        if let None = value.value {
            return DbValueEnum::None(false);
        }
        value.value.unwrap()
    }
}
