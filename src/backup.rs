use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    process,
};

use nix::{
    sys::wait::{WaitPidFlag, WaitStatus, waitpid},
    unistd::{ForkResult, fork},
};

use crate::{config, kv_store::KVStore};

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
    let temp_path = format!("{}.tmp", config::BACKUP_FILE);
    let mut temp_file = File::create(&temp_path)?;

    rmp_serde::encode::write(&mut temp_file, &data)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    temp_file.flush()?;

    // Atomically rename the temporary file to the actual backup file
    fs::rename(temp_path, config::BACKUP_FILE)?;

    Ok(())
}

pub fn restore_from_backup(store: &KVStore) -> io::Result<()> {
    if !Path::new(config::BACKUP_FILE).exists() {
        println!("No backup file found");
        return Ok(());
    }

    let file = File::open(config::BACKUP_FILE)?;
    let data: HashMap<String, String> =
        rmp_serde::decode::from_read(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    store.restore_from_backup(data);
    println!("Data restored from backup");

    Ok(())
}
