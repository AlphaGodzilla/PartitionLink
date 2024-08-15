use ahash::AHashMap;

pub enum DbValue {
    String(String),
    Bytes(Vec<u8>),
    List(Vec<DbValue>),
    Hash(AHashMap<String, DbValue>),
}

pub struct Database {
    db: AHashMap<String, DbValue>,
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

    fn str_set(&mut self, key: String, value: DbValue) {
        self.db.insert(key, value);
    }
}
