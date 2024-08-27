use ahash::AHashMap;

pub enum DBValue {
    String(String),
    Bytes(Vec<u8>),
    List(Vec<DBValue>),
    Hash(AHashMap<String, DBValue>),
}

pub struct Database {
    db: AHashMap<String, DBValue>,
}

impl Default for Database {
    fn default() -> Self {
        Database::new()
    }
}

impl Database {
    fn new() -> Self {
        Database {
            db: AHashMap::new(),
        }
    }

    fn str_set(&mut self, key: String, value: DBValue) {
        self.db.insert(key, value);
    }

    fn str_get(&mut self, key: &str) -> Option<&DBValue> {
        self.db.get(key)
    }
}
