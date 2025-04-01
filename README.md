# SimpleKV

A simple Redis-like key-value store implementation in Rust. This project provides a basic TCP server that supports fundamental key-value operations with a Redis-compatible interface.

## Features

- In-memory key-value store
- Multi-client support with concurrent connections
- Redis-like command interface
- Simple text-based protocol
- Advanced data persistence
  - MessagePack-based transaction log (AOF-like)
  - Periodic snapshot backup (every 60 seconds)
  - Automatic recovery using both snapshot and transaction logs
  - Automatic log rotation to prevent unbounded growth

## Running

```bash
cargo run
```

The server will start listening on `localhost:6379` (the default Redis port).

## Testing with Command Line

You can test the server using the `nc` (netcat) command:

```bash
nc localhost 6379
```

### Supported Commands

1. **SET key value**
   - Sets the value for a key
   - Response: "OK" on success
   - Each operation is logged to the transaction log

```
SET mykey hello
```

2. **GET key**
   - Retrieves the value for a key
   - Response: value if found, "(nil)" if key doesn't exist

```
GET mykey
```

3. **DEL key**
   - Deletes a key and its value
   - Response: "1" if key was deleted, "0" if key didn't exist
   - The deletion is logged to the transaction log

```
DEL mykey
```

### Example Session

```
> SET mykey hello
OK
> GET mykey
hello
> GET nonexistent
(nil)
> DEL mykey
1
> GET mykey
(nil)
```

## Implementation Details

- Uses `std::sync::Arc` and `Mutex` for thread-safe state management
- Implements a multi-threaded TCP server using `std::net::TcpListener`
- Each client connection is handled in a separate thread
- Uses a simple text-based protocol with newline-terminated commands

### Persistence Strategy

The server implements a hybrid persistence strategy:

1. **Transaction Log (AOF-like)**
   - Each write operation (SET/DEL) is logged immediately
   - Uses MessagePack format for efficient storage
   - Logs are automatically rotated when they exceed 1MB
   - Located in the `txlogs` directory

2. **Snapshot Backup**
   - Full state snapshot every 60 seconds
   - Uses MessagePack format for efficient storage and recovery
   - Atomic updates using temporary files
   - Located at `kv_store_backup.mp`

3. **Recovery Process**
   - Loads the latest snapshot if available
   - Replays transaction logs to recover any changes since the last snapshot
   - Ensures consistency by applying operations in order

## Error Handling

The server provides error messages for:

- Invalid command format
- Wrong number of arguments
- Unknown commands
- File system errors during persistence operations

## Limitations

- Limited command set (SET/GET/DEL only)
- No support for data types other than strings
- Basic persistence without log compaction
