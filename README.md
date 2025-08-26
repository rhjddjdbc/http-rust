# Rust HTTP Server

This project is a simple HTTP server implemented in Rust, created as a **learning project** to explore and demonstrate core concepts of HTTP servers, including request parsing, routing, rate limiting, and concurrency.

> **Disclaimer:**
> This server is designed for educational purposes only. It is **not** production-ready and should **not** be used as-is in real-world applications or exposed to the public internet.
> Use it as a foundation to learn from and build upon, but always consider security, robustness, and scalability when developing production software.

## Features

* Basic HTTP methods supported: `GET`, `POST`, `PUT`, `DELETE`, `OPTIONS`
* Rate limiting per client IP to prevent abuse
* Serving static files securely from a `public` directory
* Basic JSON API endpoint at `/api/status`
* Placeholder support for multipart/form-data uploads
* Simple cookie handling
* Thread pool for concurrent connection handling
* Configurable via a TOML config file

## Configuration

The server is configured through a `config.toml` file in the project root. You can specify:

* Host (IP or hostname to bind to)
* Port number
* Number of threads for the thread pool
* Rate limiter settings (enable/disable, time window in seconds, max requests)

Example `config.toml`:

```toml
host = "127.0.0.1"
port = 8080
threads = 4

[rate_limiter]
enabled = true
window_seconds = 60
max_requests = 100
```

## Serving Static Files

The server serves files from the `public` directory relative to the current working directory. A simple example HTML page `index.html` is included in this folder as the default landing page. You can customize or add more static assets there.

## Usage

1. **Build** the project:

   ```bash
   cargo build --release
   ```

2. **Run** the server:

   ```bash
   ./target/release/rust-http-server
   ```

3. Open a browser and visit `http://<host>:<port>` (e.g., `http://127.0.0.1:8080`).

## Notes

* The rate limiter works by tracking requests per IP address in a rolling time window.
* Request parsing is basic and handles standard HTTP/1.1 requests.
* Multipart/form-data upload support is only a placeholder and not fully implemented.
* The thread pool manages worker threads to allow multiple simultaneous connections.
* This code is structured in modules for connection handling, parsing, routing, response generation, and utilities.

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
