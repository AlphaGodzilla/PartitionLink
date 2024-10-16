use std::fmt::Display;

use crate::command::proto::{
    proto_db_value, ProtoBooleanDbValue, ProtoBytesDbValue, ProtoDbValue, ProtoHashDbValue,
    ProtoListDbValue, ProtoStringDbValue,
};
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

impl From<DBValue> for ProtoDbValue {
    fn from(value: DBValue) -> Self {
        value.to_protobuf()
    }
}

impl DBValue {
    pub fn to_protobuf(&self) -> ProtoDbValue {
        let value = match self {
            DBValue::None => Some(proto_db_value::Value::NoneDbValue(false)),
            DBValue::Boolean(v) => {
                Some(proto_db_value::Value::BooleanDbValue(ProtoBooleanDbValue {
                    value: v.clone(),
                }))
            }
            DBValue::String(value) => {
                Some(proto_db_value::Value::StringDbValue(ProtoStringDbValue {
                    value: value.clone(),
                }))
            }
            DBValue::Bytes(value) => Some(proto_db_value::Value::BytesDbValue(ProtoBytesDbValue {
                value: value.clone(),
            })),
            DBValue::List(values) => {
                let values = values.iter().map(|x| x.to_protobuf()).collect();
                Some(proto_db_value::Value::ListDbValue(ProtoListDbValue {
                    values,
                }))
            }
            DBValue::Hash(values) => {
                let values = values
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_protobuf()))
                    .collect();
                Some(proto_db_value::Value::HashDbValue(ProtoHashDbValue {
                    values,
                }))
            }
        };
        ProtoDbValue { value }
    }
}

impl From<ProtoDbValue> for DBValue {
    fn from(value: ProtoDbValue) -> Self {
        if let None = value.value {
            return DBValue::None;
        }
        let value = value.value.unwrap();
        match value {
            proto_db_value::Value::NoneDbValue(_) => DBValue::None,
            proto_db_value::Value::BooleanDbValue(b) => DBValue::Boolean(b.value),
            proto_db_value::Value::StringDbValue(s) => DBValue::String(s.value),
            proto_db_value::Value::BytesDbValue(b) => DBValue::Bytes(b.value),
            proto_db_value::Value::ListDbValue(l) => {
                let l: Vec<DBValue> = l.values.into_iter().map(|x| x.into()).collect();
                DBValue::List(l)
            }
            proto_db_value::Value::HashDbValue(h) => {
                let h: AHashMap<String, DBValue> =
                    h.values.into_iter().map(|(k, v)| (k, v.into())).collect();
                DBValue::Hash(h)
            }
        }
    }
}

impl From<ProtoDbValue> for proto_db_value::Value {
    fn from(value: ProtoDbValue) -> Self {
        if let None = value.value {
            return proto_db_value::Value::NoneDbValue(false);
        }
        value.value.unwrap()
    }
}
