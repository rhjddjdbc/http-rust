use std::collections::HashMap;
use std::io::{Read, Write};

use crate::server::response::send_html_error;

/// Parses an HTTP request from the stream.
///
/// Returns a tuple with (method, path, headers, body) or an error.
pub fn parse_request<S: Read + Write>(
    stream: &mut S,
    max_body_size: usize,
) -> Result<(String, String, HashMap<String, String>, String), ()> {
    let mut buffer = Vec::new();
    let mut temp = [0; 512];

    // Read until header end (empty line "\r\n\r\n")
    while !buffer.windows(4).any(|w| w == b"\r\n\r\n") {
        let n = stream.read(&mut temp).map_err(|_| ())?;
        if n == 0 {
            return Err(());
        }
        buffer.extend_from_slice(&temp[..n]);

        // Limit total request size (headers + body)
        if buffer.len() > max_body_size + 8192 {
            send_html_error(stream, 413, "Payload Too Large", "The request is too large.");
            return Err(());
        }
    }

    let header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
    let header_str = String::from_utf8_lossy(&buffer[..header_end]);
    let mut lines = header_str.lines();

    let request_line = lines.next().ok_or(())?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let mut headers = HashMap::new();
    for line in lines {
        if let Some((key, value)) = line.split_once(":") {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }

    let content_length = headers
        .get("content-length")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    let mut body = buffer[header_end..].to_vec();
    while body.len() < content_length {
        let n = stream.read(&mut temp).map_err(|_| ())?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&temp[..n]);
        if body.len() > max_body_size {
            send_html_error(stream, 413, "Payload Too Large", "The request is too large.");
            return Err(());
        }
    }

    if body.len() != content_length {
        send_html_error(stream, 400, "Bad Request", "Incomplete body");
        return Err(());
    }

    let body_str = String::from_utf8(body).unwrap_or_default();
    Ok((method, path, headers, body_str))
}

