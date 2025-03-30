use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// KVStore is the core data structure that holds key-value pairs
pub struct KVStore {
    store: Arc<Mutex<HashMap<String, String>>>,
}

impl KVStore {
    pub fn new() -> Self {
        KVStore {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    // Implementation of SET key value command
    pub fn set(&self, key: String, value: String) -> String {
        let mut store = self.store.lock().unwrap();
        store.insert(key, value);
        "OK".to_string()
    }

    // Implementation of GET key command
    pub fn get(&self, key: &str) -> String {
        let store = self.store.lock().unwrap();
        match store.get(key) {
            Some(value) => value.to_string(),
            None => "(nil)".to_string(),
        }
    }

    // Implementation of DEL key command
    pub fn del(&self, key: &str) -> String {
        let mut store = self.store.lock().unwrap();
        match store.remove(key) {
            Some(_) => "1".to_string(),
            None => "0".to_string(),
        }
    }
}
