use std::{net::TcpListener, sync::Arc, thread};

mod backup;
mod handle_client;
mod kv_store;

use backup::{restore_from_backup, start_backup_service};

use handle_client::handle_client;
use kv_store::KVStore;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Server listening on port 6379");

    let store = Arc::new(KVStore::new());

    if let Err(e) = restore_from_backup(&store) {
        eprintln!("Failed to restore from backup: {}", e);
    }

    start_backup_service(Arc::clone(&store));

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
