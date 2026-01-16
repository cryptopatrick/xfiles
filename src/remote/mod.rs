//! Remote API adapter layer
//!
//! This module handles communication with Twitter API,
//! including rate limiting and retry logic.

pub mod twitter;
pub mod mock;
pub mod rate_limit;
pub mod retry;

pub use twitter::{TwitterAdapter, RemoteAdapter};
pub use mock::MockAdapter;
