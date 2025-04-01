use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufWriter, Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rmp_serde::Serializer;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize, Deserialize)]
pub enum Command {
    Set { key: String, value: String },
    Del { key: String },
}

#[derive(Serialize, Deserialize)]
pub struct TransactionLog {
    timestamp: u64,
    command: Command,
}

pub struct TransactionLogger {
    current_file: BufWriter<File>,
    current_size: usize,
}

impl TransactionLogger {
    pub fn new() -> io::Result<Self> {
        fs::create_dir_all(config::TRANSACTION_LOG_DIR)?;
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let file_path = format!(
            "{}/{}{}.mp",
            config::TRANSACTION_LOG_DIR,
            config::TRANSACTION_LOG_FILE_PREFIX,
            current_time
        );

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        // 既存のファイルサイズを取得して初期化
        let metadata = file.metadata()?;
        Ok(TransactionLogger {
            current_file: BufWriter::new(file),
            current_size: metadata.len() as usize,
        })
    }

    fn write_log(&mut self, command: Command) -> io::Result<()> {
        let log = TransactionLog {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            command,
        };

        // First write the length of the serialized data as u32 (4 bytes)
        let mut buf = Vec::new();
        log.serialize(&mut Serializer::new(&mut buf))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let len = buf.len() as u32;
        println!(
            "Writing log record: size = {} bytes, current_size = {} bytes",
            len, self.current_size
        );

        self.current_file.write_all(&len.to_be_bytes())?;
        self.current_file.write_all(&buf)?;
        self.current_file.flush()?;

        self.current_size += 4 + buf.len(); // 4 bytes for length + data
        println!("Updated current_size = {} bytes", self.current_size);

        if self.current_size >= config::MAX_TRANSACTION_LOG_SIZE {
            self.rotate_log()?;
        }

        Ok(())
    }

    pub fn log_set(&mut self, key: String, value: String) -> io::Result<()> {
        self.write_log(Command::Set { key, value })
    }

    pub fn log_del(&mut self, key: String) -> io::Result<()> {
        self.write_log(Command::Del { key })
    }

    fn rotate_log(&mut self) -> io::Result<()> {
        self.current_file.flush()?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let new_file_path = format!(
            "{}/{}{}.mp",
            config::TRANSACTION_LOG_DIR,
            config::TRANSACTION_LOG_FILE_PREFIX,
            current_time
        );

        let new_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_file_path)?;

        self.current_file = BufWriter::new(new_file);
        self.current_size = 0;
        println!("Rotated log file. New file path: {}", new_file_path);

        Ok(())
    }

    pub fn apply_logs<P: AsRef<Path>>(dir: P, mut apply_fn: impl FnMut(Command)) -> io::Result<()> {
        let dir = dir.as_ref();
        println!("Scanning directory: {:?}", dir);
        if !dir.exists() {
            println!("Directory does not exist");
            return Ok(());
        }

        println!("Reading directory entries...");
        let mut entries: Vec<_> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "mp")
                    .unwrap_or(false)
            })
            .collect();

        println!("Found {} entries", entries.len());
        entries.sort_by_key(|entry| entry.path());

        for entry in entries {
            let path = entry.path();
            println!("Processing file: {:?}", path);
            let mut file = io::BufReader::new(File::open(&path)?);
            let mut len_bytes = [0u8; 4];

            let mut record_count = 0;
            loop {
                match file.read_exact(&mut len_bytes) {
                    Ok(()) => {
                        let len = u32::from_be_bytes(len_bytes) as usize;
                        println!(
                            "Reading record #{} (length: {} bytes)",
                            record_count + 1,
                            len
                        );

                        const MAX_RECORD_SIZE: usize = 1024 * 1024; // 1MB
                        if len > MAX_RECORD_SIZE {
                            println!(
                                "Record size too large: {} bytes (max: {} bytes)",
                                len, MAX_RECORD_SIZE
                            );
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("Record size too large: {} bytes", len),
                            ));
                        }

                        let mut buf = vec![0u8; len];
                        match file.read_exact(&mut buf) {
                            Ok(()) => {
                                match TransactionLog::deserialize(
                                    &mut rmp_serde::decode::Deserializer::new(&*buf),
                                ) {
                                    Ok(log) => {
                                        let cmd_desc = match &log.command {
                                            Command::Set { key, value } => {
                                                format!("SET {} = {}", key, value)
                                            }
                                            Command::Del { key } => format!("DEL {}", key),
                                        };
                                        println!(
                                            "Successfully deserialized record #{}: {}",
                                            record_count + 1,
                                            cmd_desc
                                        );
                                        apply_fn(log.command);
                                        record_count += 1;
                                    }
                                    Err(e) => {
                                        println!(
                                            "Failed to deserialize record #{}: {}",
                                            record_count + 1,
                                            e
                                        );
                                        return Err(io::Error::new(io::ErrorKind::Other, e));
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Failed to read record #{} data: {}", record_count + 1, e);
                                return Err(e);
                            }
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                        println!("Reached end of file after {} records", record_count);
                        break;
                    }
                    Err(e) => {
                        println!("Error reading record length: {}", e);
                        return Err(e);
                    }
                }
            }
            println!(
                "Finished processing file: {:?} ({} records)",
                path, record_count
            );
        }

        Ok(())
    }
}
