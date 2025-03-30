use std::{net::TcpListener, sync::Arc, thread};

mod backup;
mod handle_client;
mod kv_store;

use backup::{restore_from_backup, start_backup_service};

use handle_client::handle_client;
use kv_store::KVStore;

fn main() {
    // Start TCP server on port 6379 (same as Redis default port)
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Server listening on port 6379");

    let store = Arc::new(KVStore::new());

    // Try to restore data from backup if exists
    if let Err(e) = restore_from_backup(&store) {
        eprintln!("Failed to restore from backup: {}", e);
    }

    // Start backup service
    start_backup_service(Arc::clone(&store));

    // Accept incoming client connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = Arc::clone(&store);
                thread::spawn(move || {
                    handle_client(stream, store);
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
