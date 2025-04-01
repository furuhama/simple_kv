use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

use crate::transaction_log::TransactionLogger;

// KVStore is the core data structure that holds key-value pairs
pub struct KVStore {
    store: Arc<Mutex<HashMap<String, String>>>,
    logger: Arc<Mutex<TransactionLogger>>,
    should_log: bool, // トランザクションをログに記録するかどうか
}

impl KVStore {
    // Get all data from the store for backup
    pub fn get_all_data(&self) -> HashMap<String, String> {
        let store = self.store.lock().unwrap();
        store.clone()
    }

    // Restore data from backup
    pub fn restore_from_backup(&self, data: HashMap<String, String>) {
        let mut store = self.store.lock().unwrap();
        *store = data;
    }

    // トランザクションログの記録を一時的に無効化
    pub fn disable_logging(&mut self) {
        self.should_log = false;
    }

    // トランザクションログの記録を再開
    pub fn enable_logging(&mut self) {
        self.should_log = true;
    }
}

impl KVStore {
    pub fn new() -> io::Result<Self> {
        Ok(KVStore {
            store: Arc::new(Mutex::new(HashMap::new())),
            logger: Arc::new(Mutex::new(TransactionLogger::new()?)),
            should_log: true,
        })
    }

    // Implementation of SET key value command
    pub fn set(&self, key: String, value: String) -> io::Result<String> {
        let mut store = self.store.lock().unwrap();
        store.insert(key.clone(), value.clone());

        if self.should_log {
            let mut logger = self.logger.lock().unwrap();
            logger.log_set(key, value)?;
        }

        Ok("OK".to_string())
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
    pub fn del(&self, key: &str) -> io::Result<String> {
        let mut store = self.store.lock().unwrap();
        let result = match store.remove(key) {
            Some(_) => {
                if self.should_log {
                    let mut logger = self.logger.lock().unwrap();
                    logger.log_del(key.to_string())?;
                }
                "1".to_string()
            }
            None => "0".to_string(),
        };
        Ok(result)
    }
}
