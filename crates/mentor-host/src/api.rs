//! Minimal client for the Firecracker API: JSON requests over HTTP/1.1 on a Unix socket.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

/// Sends one request and returns the response body. A status outside 2xx is an error carrying the body.
pub fn call(socket: &Path, method: &str, path: &str, body: &serde_json::Value) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(30)))?;
    let body = body.to_string();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nAccept: application/json\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    // Firecracker keeps the connection open: read the headers, then exactly `Content-Length` bytes.
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line)?;
    let status: u16 = status_line.split_whitespace().nth(1).and_then(|code| code.parse().ok()).unwrap_or(0);
    let mut length = 0usize;
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header)? == 0 || header.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.eq_ignore_ascii_case("content-length") {
                length = value.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut payload = vec![0u8; length];
    reader.read_exact(&mut payload)?;
    let payload = String::from_utf8_lossy(&payload).into_owned();
    if (200..300).contains(&status) {
        Ok(payload)
    } else {
        Err(std::io::Error::other(format!("Firecracker API {method} {path}: HTTP {status}: {payload}")))
    }
}
