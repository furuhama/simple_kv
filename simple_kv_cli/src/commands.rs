use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;

use crate::client::Client;
use crate::error::{ClientError, Result};
use chrono::{DateTime, Local};
use prettytable::{Table, row};
use rustyline::DefaultEditor;
use serde::Deserialize;

#[derive(Deserialize)]
struct TransactionLog {
    timestamp: u64,
    command: Command,
}

#[derive(Deserialize)]
enum Command {
    Set { key: String, value: String },
    Del { key: String },
}

pub fn start_console(host: &str, port: u16) -> Result<()> {
    let mut client = Client::connect(host, port)?;
    println!("Connected to simple_kv server at {}:{}", host, port);

    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;

                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                if trimmed.eq_ignore_ascii_case("quit") {
                    println!("Goodbye!");
                    break;
                }

                if trimmed.eq_ignore_ascii_case("help") {
                    print_help();
                    continue;
                }

                match client.execute_command(trimmed) {
                    Ok(response) => println!("{}", response),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(_) => {
                println!("\nGoodbye!");
                break;
            }
        }
    }

    Ok(())
}

pub fn read_transaction_log(file_path: PathBuf) -> Result<()> {
    let file = File::open(&file_path).map_err(ClientError::File)?;
    let mut reader = BufReader::new(file);

    let mut buf = Vec::new();
    let mut len_bytes = [0u8; 4];

    loop {
        match reader.read_exact(&mut len_bytes) {
            Ok(()) => {
                let len = u32::from_be_bytes(len_bytes) as usize;
                buf.resize(len, 0);

                reader.read_exact(&mut buf).map_err(ClientError::File)?;

                let log: TransactionLog = rmp_serde::decode::from_slice(&buf)?;
                let datetime: DateTime<Local> = DateTime::from_timestamp(log.timestamp as i64, 0)
                    .unwrap_or_default()
                    .into();

                match log.command {
                    Command::Set { key, value } => {
                        println!(
                            "[{}] SET {} = {}",
                            datetime.format("%Y-%m-%d %H:%M:%S"),
                            key,
                            value
                        );
                    }
                    Command::Del { key } => {
                        println!("[{}] DEL {}", datetime.format("%Y-%m-%d %H:%M:%S"), key);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(ClientError::File(e)),
        }
    }

    Ok(())
}

pub fn read_backup(file_path: PathBuf) -> Result<()> {
    let file = File::open(&file_path).map_err(ClientError::File)?;
    let data: std::collections::HashMap<String, String> = rmp_serde::from_read(file)?;

    let mut table = Table::new();
    table.add_row(row!["Key", "Value"]);

    for (key, value) in data {
        table.add_row(row![key, value]);
    }

    table.printstd();
    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  SET <key> <value>  Set a key-value pair");
    println!("  GET <key>         Get the value for a key");
    println!("  DEL <key>         Delete a key-value pair");
    println!("  HELP              Show this help message");
    println!("  QUIT              Exit the console");
}
