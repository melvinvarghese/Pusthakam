//! A bounded, single-request HTTP/1 connection. No filesystem routing or proxy forwarding.
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};

pub const MAX_BODY: usize = 140 * 1024 * 1024;
const MAX_HEADERS: usize = 16 * 1024;

pub struct Head {
    pub method: String,
    pub target: String,
    pub headers: BTreeMap<String, String>,
    pub length: usize,
    pub raw: String,
    buffered: Vec<u8>,
}
impl Head {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("")
    }
}

pub fn read_head(stream: &mut TcpStream) -> Result<Head, u16> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let start = Instant::now();
    let mut bytes = Vec::new();
    let boundary;
    loop {
        if let Some(position) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            boundary = position;
            break;
        }
        if bytes.len() > MAX_HEADERS || start.elapsed() > Duration::from_secs(10) {
            return Err(431);
        }
        let mut chunk = [0u8; 4096];
        let count = stream.read(&mut chunk).map_err(|_| 408u16)?;
        if count == 0 {
            return Err(400);
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    if boundary > MAX_HEADERS {
        return Err(431);
    }
    let raw = std::str::from_utf8(&bytes[..boundary])
        .map_err(|_| 400u16)?
        .to_string();
    let mut lines = raw.split("\r\n");
    let request: Vec<_> = lines.next().unwrap_or("").split_whitespace().collect();
    if request.len() != 3
        || !["HTTP/1.1", "HTTP/1.0"].contains(&request[2])
        || !request[1].starts_with('/')
        || request[1].contains('#')
    {
        return Err(400);
    }
    let method = request[0].to_string();
    let target = request[1].to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        let (name, value) = line.split_once(':').ok_or(400u16)?;
        if name.is_empty()
            || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            || value.bytes().any(|b| b < 32 && b != b'\t' || b == 127)
        {
            return Err(400);
        }
        if headers
            .insert(name.to_ascii_lowercase(), value.trim().to_string())
            .is_some()
        {
            return Err(400);
        }
    }
    if !headers.contains_key("host") || headers.contains_key("transfer-encoding") {
        return Err(400);
    }
    if headers.contains_key("expect") {
        return Err(417);
    }
    let length = headers
        .get("content-length")
        .map(|value| {
            if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                return Err(400u16);
            }
            value.parse::<usize>().map_err(|_| 413u16)
        })
        .transpose()?
        .unwrap_or(0);
    if length > MAX_BODY {
        return Err(413);
    }
    if method != "POST" && length != 0 {
        return Err(400);
    }
    Ok(Head {
        method,
        target,
        headers,
        length,
        raw,
        buffered: bytes[boundary + 4..].to_vec(),
    })
}

pub fn read_body(stream: &mut TcpStream, head: &Head, limit: usize) -> Result<Vec<u8>, u16> {
    if head.length > limit {
        return Err(413);
    }
    let start = Instant::now();
    let mut body = Vec::with_capacity(head.length.min(1024 * 1024));
    body.extend_from_slice(&head.buffered[..head.buffered.len().min(head.length)]);
    while body.len() < head.length {
        if start.elapsed() > Duration::from_secs(if limit > 16 * 1024 { 15 * 60 } else { 30 }) {
            return Err(408);
        }
        let mut chunk = [0u8; 32 * 1024];
        let available = (head.length - body.len()).min(chunk.len());
        let count = stream.read(&mut chunk[..available]).map_err(|_| 408u16)?;
        if count == 0 {
            return Err(400);
        }
        body.extend_from_slice(&chunk[..count]);
    }
    Ok(body)
}

pub fn respond(
    stream: &mut TcpStream,
    status: u16,
    mime: &str,
    body: &[u8],
    headers: &[(&str, String)],
) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Content Too Large",
        417 => "Expectation Failed",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        503 => "Service Unavailable",
        _ => "Internal Server Error",
    };
    let mut response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\n",body.len());
    for (key, value) in headers {
        if !value.contains(['\r', '\n']) {
            response.push_str(&format!("{key}: {value}\r\n"));
        }
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes())?;
    stream.write_all(body)
}
pub fn reject(stream: &mut TcpStream, status: u16) -> std::io::Result<()> {
    respond(
        stream,
        status,
        "application/json",
        b"{\"error\":{\"code\":\"HTTP_REQUEST\",\"message\":\"Request not accepted.\"}}",
        &[],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    fn parse(request: &[u8]) -> Result<(Head, Vec<u8>), u16> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let bytes = request.to_vec();
        let client = std::thread::spawn(move || {
            let mut s = TcpStream::connect(address).unwrap();
            s.write_all(&bytes).unwrap();
        });
        let (mut stream, _) = listener.accept().unwrap();
        let result = read_head(&mut stream).and_then(|head| {
            let body = read_body(&mut stream, &head, MAX_BODY)?;
            Ok((head, body))
        });
        client.join().unwrap();
        result
    }
    #[test]
    fn exact_body_and_header_parsing() {
        let (head,body)=parse(b"POST /api/staff/commands HTTP/1.1\r\nHost: localhost\r\nContent-Length: 2\r\n\r\n{}ignored-pipeline").unwrap();
        assert_eq!(head.header("host"), Some("localhost"));
        assert_eq!(body, b"{}");
    }
    #[test]
    fn rejects_ambiguous_and_oversized_requests() {
        for request in [
            b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 0\r\nContent-Length: 1\r\n\r\n"
                .as_slice(),
            b"POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n",
            b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: -1\r\n\r\n",
            b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 999999999999\r\n\r\n",
            b"GET / HTTP/1.1\r\nHost: a\r\nHost: b\r\n\r\n",
        ] {
            assert!(parse(request).is_err());
        }
    }
}
