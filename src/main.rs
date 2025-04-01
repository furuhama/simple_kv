use std::{net::TcpListener, sync::Arc, thread, time::Instant};

mod backup;
mod config;
mod handle_client;
mod kv_store;
mod transaction_log;

use backup::{execute_backup, restore_data};
use handle_client::handle_client;
use kv_store::KVStore;

fn main() {
    let addr = format!("{}:{}", config::SERVER_HOST, config::SERVER_PORT);
    let listener = TcpListener::bind(&addr).unwrap();
    println!("Server listening on {}", addr);

    // KVStoreを作成し、バックアップとトランザクションログを適用
    let mut store = KVStore::new().expect("Failed to initialize KVStore");
    if let Err(e) = restore_data(&mut store) {
        eprintln!("Failed to restore data: {}", e);
    }

    // Arc化してマルチスレッド対応に
    let store = Arc::new(store);
    let backup_interval = config::BACKUP_INTERVAL;
    let mut last_backup = Instant::now();

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
                thread::sleep(config::CONNECTION_RETRY_INTERVAL);
                continue;
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
