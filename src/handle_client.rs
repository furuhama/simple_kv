use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    sync::Arc,
};

use crate::kv_store::KVStore;

pub fn handle_client(mut stream: TcpStream, store: Arc<KVStore>) {
    let addr = stream.peer_addr().unwrap();
    println!("New client connected: {}", addr);

    if let Err(e) = stream.set_nonblocking(false) {
        eprintln!("Error setting stream to blocking mode: {}", e);
        return;
    }

    let reader = match stream.try_clone() {
        Ok(stream) => BufReader::new(stream),
        Err(e) => {
            eprintln!("Error cloning stream: {}", e);
            return;
        }
    };

    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!("Error reading line from {}: {}", addr, e);
                break;
            }
        };

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        println!("Received command from {}: {}", addr, line);

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

        if let Err(e) = stream.write_all(format!("{}\n", response).as_bytes()) {
            eprintln!("Error writing to {}: {}", addr, e);
            break;
        }
        if let Err(e) = stream.flush() {
            eprintln!("Error flushing stream for {}: {}", addr, e);
            break;
        }

        println!("Response to {}: {}", addr, response);
    }

    println!("Client disconnected: {}", addr);
}
