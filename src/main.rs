mod server;
mod config;

use std::fs;
use log::{error, info};
use std::net::TcpListener;
use std::sync::Arc;

use crate::server::connection::handle_http_connection;
use crate::server::rate_limiter::RateLimiter;
use crate::server::thread_pool::ThreadPool;
use crate::config::Config;

fn main() {
    // Initialize logger (env_logger expects RUST_LOG=info|error|debug etc. as env var)
    env_logger::init();

    // Load config
    let config_str = fs::read_to_string("config.toml")
        .expect("Could not read config.toml");
    let config: Config = toml::from_str(&config_str)
        .expect("Error parsing configuration");

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr)
        .expect("Could not open port");

    // Thread pool with configurable thread count
    let pool = ThreadPool::new(config.threads);

    // Rate limiter wrapped in Arc for thread safety
    let rate_limiter = Arc::new(RateLimiter::new_with_limits(
        config.rate_limiter.window_seconds,
        config.rate_limiter.max_requests,
    ));

    info!("HTTP server started at http://{}", addr);

    for stream in listener.incoming() {
        let rate_limiter = Arc::clone(&rate_limiter);

        match stream {
            Ok(stream) => {
                // Process connection in thread pool
                pool.execute(move || {
                    if let Err(e) = handle_http_connection(stream, rate_limiter) {
                        error!("Error in HTTP connection: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Error establishing connection: {}", e);
            }
        }
    }
}

