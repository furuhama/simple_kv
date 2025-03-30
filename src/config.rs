use std::time::Duration;

// Server configurations
pub const SERVER_HOST: &str = "127.0.0.1";
pub const SERVER_PORT: u16 = 6379;
pub const CONNECTION_RETRY_INTERVAL: Duration = Duration::from_millis(100);

// Backup configurations
pub const BACKUP_INTERVAL: Duration = Duration::from_secs(60);
pub const BACKUP_FILE: &str = "kv_store_backup.mp";
