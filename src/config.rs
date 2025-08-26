use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub threads: usize,
    pub rate_limiter: RateLimiterConfig,
}

#[derive(Debug, Deserialize)]
pub struct RateLimiterConfig {
    pub enabled: bool,
    pub window_seconds: u64,
    pub max_requests: usize,
}

