use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process,
    time::Duration,
};

use nix::{
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
    unistd::{ForkResult, fork},
};
use serde_json;

use crate::kv_store::KVStore;

pub const BACKUP_INTERVAL: Duration = Duration::from_secs(60);
const BACKUP_FILE: &str = "kv_store_backup.json";

pub fn execute_backup(store: &KVStore) {
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            println!("Started backup process with PID: {}", child);
            match waitpid(child, Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::StillAlive) => {
                    println!("Backup process started in background");
                }
                Ok(_) => {
                    println!("Backup process completed immediately");
                }
                Err(e) => eprintln!("Error checking backup process: {}", e),
            }
        }
        Ok(ForkResult::Child) => {
            println!("Backup process started");
            match backup_to_file(store) {
                Ok(()) => {
                    println!("Backup completed successfully");
                    process::exit(0);
                }
                Err(e) => {
                    eprintln!("Backup error: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Fork failed: {}", err);
        }
    }
}

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
