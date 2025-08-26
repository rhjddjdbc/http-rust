use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    window: Duration,
    max_requests: usize,
}

impl RateLimiter {
    pub fn new_with_limits(window_secs: u64, max_requests: usize) -> Self {
        RateLimiter {
            inner: Arc::new(Mutex::new(HashMap::new())),
            window: Duration::from_secs(window_secs),
            max_requests,
        }
    }

    pub fn is_rate_limited(&self, ip: &IpAddr) -> bool {
        let mut map = self.inner.lock().unwrap();
        let now = Instant::now();

        let timestamps = map.entry(*ip).or_insert_with(Vec::new);
        timestamps.retain(|&time| now.duration_since(time) < self.window);

        if timestamps.len() >= self.max_requests {
            true
        } else {
            timestamps.push(now);
            false
        }
    }
}

