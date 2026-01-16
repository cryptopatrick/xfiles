//! Rate limiting for Twitter API

use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Rate limiter for API calls
pub struct RateLimiter {
    /// Maximum requests per window
    max_requests: usize,
    /// Time window duration
    window: Duration,
    /// Request timestamps
    requests: Mutex<Vec<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Mutex::new(Vec::new()),
        }
    }

    /// Wait if necessary to respect rate limits, then record the request
    pub async fn acquire(&self) {
        let mut requests = self.requests.lock().await;

        // Remove expired timestamps
        let cutoff = Instant::now() - self.window;
        requests.retain(|&ts| ts > cutoff);

        // If at limit, wait until oldest request expires
        if requests.len() >= self.max_requests {
            if let Some(&oldest) = requests.first() {
                let wait_time = self.window - (Instant::now() - oldest);
                if !wait_time.is_zero() {
                    drop(requests); // Release lock while waiting
                    tokio::time::sleep(wait_time).await;
                    requests = self.requests.lock().await;
                    requests.retain(|&ts| ts > Instant::now() - self.window);
                }
            }
        }

        // Record this request
        requests.push(Instant::now());
    }

    /// Check if we can make a request without waiting
    pub async fn can_proceed(&self) -> bool {
        let mut requests = self.requests.lock().await;
        let cutoff = Instant::now() - self.window;
        requests.retain(|&ts| ts > cutoff);
        requests.len() < self.max_requests
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, Duration::from_millis(100));

        // First two requests should succeed immediately
        limiter.acquire().await;
        limiter.acquire().await;

        // Third request should wait
        let start = Instant::now();
        limiter.acquire().await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(90)); // Allow some slack
    }
}
