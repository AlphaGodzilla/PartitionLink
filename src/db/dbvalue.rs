use std::fmt::Display;

use crate::command::proto;
use crate::command::proto::out::db_value;
use crate::command::proto::out::{BytesDbValue, HashDbValue, ListDbValue, StringDbValue};
use ahash::{AHashMap, HashMap};

#[derive(Clone)]
pub enum DBValue {
    None,
    String(String),
    Bytes(Vec<u8>),
    List(Vec<DBValue>),
    Hash(AHashMap<String, DBValue>),
}

impl Display for DBValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_name = match self {
            Self::None => "None",
            Self::String(_) => "String",
            Self::Bytes(_) => "Bytes",
            Self::List(_) => "List",
            Self::Hash(_) => "Hash",
        };
        write!(f, "DBValue::{}", type_name)
    }
}

impl DBValue {
    pub fn to_protobuf(&self) -> crate::command::proto::out::DbValue {
        let value = match self {
            DBValue::None => Some(db_value::Value::NoneDbValue(false)),
            DBValue::String(value) => Some(db_value::Value::StringDbValue(StringDbValue {
                value: value.clone(),
            })),
            DBValue::Bytes(value) => Some(db_value::Value::BytesDbValue(BytesDbValue {
                value: value.clone(),
            })),
            DBValue::List(values) => {
                let values = values.iter().map(|x| x.to_protobuf()).collect();
                Some(db_value::Value::ListDbValue(ListDbValue { values }))
            }
            DBValue::Hash(values) => {
                let values = values
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_protobuf()))
                    .collect();
                Some(db_value::Value::HashDbValue(HashDbValue { values }))
            }
        };
        crate::command::proto::out::DbValue { value }
    }
}

impl From<proto::out::DbValue> for DBValue {
    fn from(value: crate::command::proto::out::DbValue) -> Self {
        if let None = value.value {
            return DBValue::None;
        }
        let value = value.value.unwrap();
        match value {
            proto::out::db_value::Value::NoneDbValue(_) => DBValue::None,
            proto::out::db_value::Value::StringDbValue(s) => DBValue::String(s.value),
            proto::out::db_value::Value::BytesDbValue(b) => DBValue::Bytes(b.value),
            proto::out::db_value::Value::ListDbValue(l) => {
                let l: Vec<DBValue> = l.values.into_iter().map(|x| x.into()).collect();
                DBValue::List(l)
            }
            proto::out::db_value::Value::HashDbValue(h) => {
                let h: AHashMap<String, DBValue> =
                    h.values.into_iter().map(|(k, v)| (k, v.into())).collect();
                DBValue::Hash(h)
            }
        }
    }
}

impl From<proto::out::DbValue> for proto::out::db_value::Value {
    fn from(value: proto::out::DbValue) -> Self {
        if let None = value.value {
            return proto::out::db_value::Value::NoneDbValue(false);
        }
        let value = value.value.unwrap();
        match value {
            proto::out::db_value::Value::NoneDbValue(v) => {
                proto::out::db_value::Value::NoneDbValue(v)
            }
            proto::out::db_value::Value::StringDbValue(v) => {
                proto::out::db_value::Value::StringDbValue(v)
            }
            proto::out::db_value::Value::BytesDbValue(v) => {
                proto::out::db_value::Value::BytesDbValue(v)
            }
            proto::out::db_value::Value::ListDbValue(v) => {
                proto::out::db_value::Value::ListDbValue(v)
            }
            proto::out::db_value::Value::HashDbValue(v) => {
                proto::out::db_value::Value::HashDbValue(v)
            }
        }
    }
}
