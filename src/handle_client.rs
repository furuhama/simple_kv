use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::Arc,
};

use crate::kv_store::KVStore;

// Function to handle client connections
pub fn handle_client(mut stream: TcpStream, store: Arc<KVStore>) {
    let reader = BufReader::new(stream.try_clone().unwrap());

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        let response = match parts[0].to_uppercase().as_str() {
            "SET" => {
                if parts.len() != 3 {
                    "ERROR: SET command requires KEY and VALUE".to_string()
                } else {
                    store.set(parts[1].to_string(), parts[2].to_string())
                }
            }
            "GET" => {
                if parts.len() != 2 {
                    "ERROR: GET command requires KEY".to_string()
                } else {
                    store.get(parts[1])
                }
            }
            "DEL" => {
                if parts.len() != 2 {
                    "ERROR: DEL command requires KEY".to_string()
                } else {
                    store.del(parts[1])
                }
            }
            _ => "ERROR: Unknown command".to_string(),
        };

        // Add newline to response and send it
        stream
            .write_all(format!("{}\n", response).as_bytes())
            .unwrap();
        stream.flush().unwrap();
    }
}
