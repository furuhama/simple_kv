# simple_kv_cli

A command-line client for simple_kv server, providing interactive console and data inspection tools.

## Features

- Interactive console for key-value operations (similar to redis-cli)
- Transaction log reader
- Backup file reader

## Usage

### Interactive Console

Start an interactive console to connect to the simple_kv server:

```bash
simple_kv_cli console [--host HOST] [--port PORT]
```

Available commands in the console:
- `SET <key> <value>`: Set a key-value pair
- `GET <key>`: Get the value for a key
- `DEL <key>`: Delete a key-value pair
- `HELP`: Show help message
- `QUIT`: Exit the console

Example:
```bash
$ simple_kv_cli console
Connected to simple_kv server at 127.0.0.1:6379
> SET mykey value1
OK
> GET mykey
"value1"
> QUIT
Goodbye!
```

### Transaction Log Reader

Read and display transaction logs in a human-readable format:

```bash
simple_kv_cli txlog --read <log-file-path>
```

Example:
```bash
$ simple_kv_cli txlog --read txlogs/txlog_1234567890.mp
[2024-04-02 15:30:00] SET user123 = John Doe
[2024-04-02 15:30:05] DEL old_key
```

### Backup Reader

Read and display backup data in a table format:

```bash
simple_kv_cli backup --read <backup-file-path>
```

Example:
```bash
$ simple_kv_cli backup --read kv_store_backup.mp
+----------+-----------+
| Key      | Value     |
+----------+-----------+
| user123  | John Doe  |
| counter  | 42        |
+----------+-----------+
```

## Command Line Options

```bash
simple_kv_cli <COMMAND>

Commands:
  console  Start interactive console
  txlog    Read transaction log
  backup   Read backup
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
