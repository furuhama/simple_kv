use rustyline::error::ReadlineError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Failed to connect to server: {0}")]
    Connection(#[from] std::io::Error),

    #[error("Failed to read file: {0}")]
    File(std::io::Error),

    #[error("Failed to decode MessagePack data: {0}")]
    Decode(#[from] rmp_serde::decode::Error),

    #[error("Readline error: {0}")]
    Readline(#[from] ReadlineError),
}

pub type Result<T> = std::result::Result<T, ClientError>;
