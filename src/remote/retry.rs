//! Retry logic with exponential backoff

use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// Retry a function with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 0;
    let mut backoff = config.initial_backoff;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempt += 1;
                if attempt >= config.max_attempts {
                    return Err(e);
                }

                sleep(backoff).await;

                // Exponential backoff
                backoff = Duration::from_secs_f64(
                    (backoff.as_secs_f64() * config.multiplier).min(config.max_backoff.as_secs_f64())
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let config = RetryConfig::default();
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(&config, || {
            let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                if count < 3 {
                    Err("temporary error")
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let config = RetryConfig {
            max_attempts: 2,
            ..Default::default()
        };
        let attempts = AtomicUsize::new(0);

        let result = retry_with_backoff(&config, || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err::<i32, _>("persistent error") }
        })
        .await;

        assert_eq!(result, Err("persistent error"));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}
