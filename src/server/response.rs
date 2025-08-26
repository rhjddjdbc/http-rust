use log::error;
use std::fs;
use std::io::Write;
use std::path::Path;
use mime_guess::from_path;

use crate::server::utils::{http_date, resolve_safe_path};

/// Handle GET requests
pub fn handle_get_request<S: Write>(stream: &mut S, path: &str) {
    let path = if path == "/" { "/index.html" } else { path };
    let resolved = resolve_safe_path(path);

    match resolved {
        Some(path) if path.is_file() => {
            match fs::read(&path) {
                Ok(contents) => {
                    let content_type = from_path(&path).first_or_octet_stream();
                    let date = http_date();

                    let headers = format!(
                        "HTTP/1.1 200 OK\r\n\
                         Content-Length: {}\r\n\
                         Content-Type: {}\r\n\
                         Date: {}\r\n\
                         Connection: close\r\n\
                         \r\n",
                        contents.len(),
                        content_type,
                        date,
                    );

                    if let Err(e) = stream.write_all(headers.as_bytes())
                        .and_then(|_| stream.write_all(&contents))
                        .and_then(|_| stream.flush()) {
                        error!("Error sending response: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error reading file {}: {}", path.display(), e);
                    send_html_error(stream, 500, "Internal Server Error", "Error reading the file");
                }
            }
        }
        _ => {
            send_404_page(stream);
        }
    }
}

/// Handle POST requests
pub fn handle_post_request<S: Write>(stream: &mut S, _path: &str, body: &str) {
    let response_body = if body.trim_start().starts_with('{') {
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(json) => format!(
                "<html><body><h1>Received JSON Data:</h1><pre>{}</pre></body></html>",
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| body.to_string())
            ),
            Err(_) => format!(
                "<html><body><h1>Received Data:</h1><pre>{}</pre></body></html>",
                html_escape::encode_text(body)
            ),
        }
    } else {
        let mut form_data = String::new();
        for pair in body.split('&') {
            let mut parts = pair.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                form_data.push_str(&format!("{}: {}\n", html_escape::encode_text(key), html_escape::encode_text(value)));
            }
        }
        format!(
            "<html><body><h1>Received Form Data:</h1><pre>{}</pre></body></html>",
            form_data
        )
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Length: {}\r\n\
         Content-Type: text/html; charset=UTF-8\r\n\
         Date: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        response_body.len(),
        http_date(),
        response_body
    );

    let _ = stream.write_all(response.as_bytes());
}

/// Handle PUT requests
pub fn handle_put_request<S: Write>(stream: &mut S, path: &str, body: &str) {
    let resolved = resolve_safe_path(path);

    if let Some(path) = resolved {
        if let Err(e) = fs::write(&path, body) {
            error!("Error writing to file {}: {}", path.display(), e);
            send_html_error(stream, 500, "Internal Server Error", "Could not write to file.");
        } else {
            let msg = format!("File {} was successfully updated.", path.display());
            send_json(stream, &serde_json::json!({ "status": "ok", "message": msg }).to_string());
        }
    } else {
        send_html_error(stream, 400, "Bad Request", "Invalid path.");
    }
}

/// Handle DELETE requests
pub fn handle_delete_request<S: Write>(stream: &mut S, path: &str) {
    let resolved = resolve_safe_path(path);

    if let Some(path) = resolved {
        if path.is_file() {
            match fs::remove_file(&path) {
                Ok(_) => {
                    send_json(stream, &serde_json::json!({ "deleted": path.to_string_lossy().to_string() }).to_string());
                }
                Err(e) => {
                    error!("Error deleting file: {}", e);
                    send_html_error(stream, 500, "Error", "File could not be deleted.");
                }
            }
        } else {
            send_html_error(stream, 404, "Not Found", "File does not exist.");
        }
    } else {
        send_html_error(stream, 400, "Bad Request", "Invalid path.");
    }
}

/// Send JSON response
pub fn send_json<S: Write>(stream: &mut S, json: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Date: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        json.len(),
        http_date(),
        json
    );
    let _ = stream.write_all(response.as_bytes());
}

/// Set a cookie (example)
pub fn send_cookie<S: Write>(stream: &mut S, cookie: &str) {
    let response = format!(
        "Set-Cookie: {}\r\n",
        cookie
    );
    let _ = stream.write_all(response.as_bytes());
}

/// Send error page
pub fn send_html_error<S: Write>(stream: &mut S, code: u16, title: &str, message: &str) {
    let date = http_date();
    let body = format!(
        "<html><body><h1>{}</h1><p>{}</p></body></html>",
        title, message
    );

    let response = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Length: {}\r\n\
         Content-Type: text/html; charset=UTF-8\r\n\
         Date: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        code,
        title,
        body.len(),
        date,
        body
    );

    let _ = stream.write_all(response.as_bytes());
}

/// Send 404 page
pub fn send_404_page<S: Write>(stream: &mut S) {
    send_html_error(stream, 404, "Not Found", "404 - File not found");
}

