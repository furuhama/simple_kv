use std::{net::TcpListener, sync::Arc, thread, time::Instant};

mod backup;
mod handle_client;
mod kv_store;

use backup::{BACKUP_INTERVAL, execute_backup, restore_from_backup};
use handle_client::handle_client;
use kv_store::KVStore;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Server listening on port 6379");

    let store = Arc::new(KVStore::new());
    let backup_interval = BACKUP_INTERVAL;
    let mut last_backup = Instant::now();

    if let Err(e) = restore_from_backup(&store) {
        eprintln!("Failed to restore from backup: {}", e);
    }

    listener.set_nonblocking(true).unwrap();

    loop {
        if last_backup.elapsed() >= backup_interval {
            execute_backup(&store);
            last_backup = Instant::now();
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    handle_client(stream, store);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
