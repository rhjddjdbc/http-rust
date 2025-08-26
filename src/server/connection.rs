use std::net::{IpAddr, TcpStream};
use std::sync::Arc;

use crate::server::rate_limiter::RateLimiter;
use crate::server::handler::handle_connection;
use crate::server::response::send_html_error;

pub fn handle_http_connection(
    mut stream: TcpStream,
    rate_limiter: Arc<RateLimiter>,
) -> Result<(), Box<dyn std::error::Error>> {
    let peer_ip = stream
        .peer_addr()
        .map(|addr| addr.ip())
        .unwrap_or_else(|_| IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));

    if rate_limiter.is_rate_limited(&peer_ip) {
        send_html_error(&mut stream, 429, "Too Many Requests", "Too many requests. Please wait a moment.");
        return Ok(());
    }

    handle_connection(&mut stream);
    Ok(())
}

