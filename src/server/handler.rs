use std::collections::HashMap;
use std::io::{Read, Write};

use crate::server::parser::parse_request;
use crate::server::response::{
    handle_get_request,
    handle_post_request,
    handle_put_request,
    handle_delete_request,
    send_html_error,
    send_json,
    send_cookie,
    send_404_page,
};

pub fn handle_connection<S: Read + Write>(stream: &mut S) {
    let max_body_size = 10 * 1024 * 1024;

    match parse_request(stream, max_body_size) {
        Ok((method, path, headers, body)) => {
            let method = method.to_uppercase();

            // Routing
            match (method.as_str(), path.as_str()) {
                // --- API JSON endpoints ---
                ("GET", "/api/status") => {
                    let json = serde_json::json!({
                        "status": "ok",
                        "time": chrono::Utc::now()
                    });
                    send_json(stream, &json.to_string());
                }

                // --- File upload with multipart/form-data ---
                ("POST", "/upload") if headers.get("content-type")
                    .map_or(false, |v| v.starts_with("multipart/form-data")) =>
                {
                    // Multipart handling can be implemented here (e.g. using the `multipart` crate)
                    let response = serde_json::json!({ "upload": "received" });
                    send_json(stream, &response.to_string());
                }

                // --- Generic handling for GET, POST, PUT, DELETE ---
                ("GET", _) => handle_get_request(stream, &path),
                ("POST", _) => handle_post_request(stream, &path, &body),
                ("PUT", _) => handle_put_request(stream, &path, &body),
                ("DELETE", _) => handle_delete_request(stream, &path),

                // --- OPTIONS for CORS or method declaration ---
                ("OPTIONS", _) => {
                    let response = "HTTP/1.1 204 No Content\r\nAllow: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n";
                    let _ = stream.write_all(response.as_bytes());
                }

                // --- Method not allowed ---
                _ => send_html_error(
                    stream,
                    405,
                    "Method Not Allowed",
                    "Only GET, POST, PUT, DELETE, OPTIONS are allowed.",
                ),
            }

            // Set cookie (example)
            if !headers.contains_key("cookie") {
                send_cookie(stream, "session_id=abc123; HttpOnly; Path=/");
            }
        }

        Err(_) => {
            send_html_error(
                stream,
                400,
                "Bad Request",
                "Error parsing the request.",
            );
        }
    }
}

