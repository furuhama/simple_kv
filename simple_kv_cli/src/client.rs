use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use crate::error::{ClientError, Result};

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect(host: &str, port: u16) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).map_err(ClientError::Connection)?;
        Ok(Client { stream })
    }

    pub fn execute_command(&mut self, command: &str) -> Result<String> {
        self.stream
            .write_all(format!("{}\n", command).as_bytes())
            .map_err(ClientError::Connection)?;
        self.stream.flush().map_err(ClientError::Connection)?;

        let mut reader = BufReader::new(&self.stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(ClientError::Connection)?;

        Ok(response.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_client_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0; 1024];
            let n = stream.read(&mut buf).unwrap();
            assert_eq!(&buf[..n], b"TEST\n");
            stream.write_all(b"OK\n").unwrap();
        });

        let mut client = Client::connect(&addr.ip().to_string(), addr.port()).unwrap();
        let response = client.execute_command("TEST").unwrap();
        assert_eq!(response, "OK");

        handle.join().unwrap();
    }
}
