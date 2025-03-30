use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    sync::Arc,
    thread,
    time::Duration,
};

use serde_json;

use crate::kv_store::KVStore;

const BACKUP_INTERVAL: Duration = Duration::from_secs(60);
const BACKUP_FILE: &str = "kv_store_backup.json";

// Function to start periodic backup
pub fn start_backup_service(store: Arc<KVStore>) {
    thread::spawn(move || {
        loop {
            if let Err(e) = backup_to_file(&store) {
                eprintln!("Backup error: {}", e);
            } else {
                println!("Backup completed successfully");
            }
            thread::sleep(BACKUP_INTERVAL);
        }
    });
}

// Function to perform the actual backup
fn backup_to_file(store: &KVStore) -> io::Result<()> {
    let data = store.get_all_data();
    let json = serde_json::to_string_pretty(&data)?;

    // Write to a temporary file first
    let temp_path = format!("{}.tmp", BACKUP_FILE);
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(json.as_bytes())?;
    temp_file.flush()?;

    // Atomically rename the temporary file to the actual backup file
    fs::rename(temp_path, BACKUP_FILE)?;

    Ok(())
}

// Function to restore data from backup file
pub fn restore_from_backup(store: &KVStore) -> io::Result<()> {
    if !Path::new(BACKUP_FILE).exists() {
        println!("No backup file found");
        return Ok(());
    }

    let content = fs::read_to_string(BACKUP_FILE)?;
    let data: HashMap<String, String> = serde_json::from_str(&content)?;
    store.restore_from_backup(data);
    println!("Data restored from backup");

    Ok(())
}
