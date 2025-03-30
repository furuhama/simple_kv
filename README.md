# SimpleKV

A simple Redis-like key-value store implementation in Rust. This project provides a basic TCP server that supports fundamental key-value operations with a Redis-compatible interface.

## Features

- In-memory key-value store
- Multi-client support with concurrent connections
- Redis-like command interface
- Simple text-based protocol

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

## Error Handling

The server provides error messages for invalid commands:

- Invalid command format
- Wrong number of arguments
- Unknown commands

## Limitations

- In-memory storage only (data is lost when server stops)
- No persistence
- Limited command set (SET/GET/DEL only)
- No support for data types other than strings
