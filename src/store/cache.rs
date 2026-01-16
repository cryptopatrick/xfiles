//! Content caching layer

use crate::dag::commit::TweetId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// In-memory cache for tweet content
pub struct ContentCache {
    cache: Arc<RwLock<HashMap<TweetId, Vec<u8>>>>,
}

impl ContentCache {
    /// Create a new content cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get content from cache
    pub fn get(&self, id: &TweetId) -> Option<Vec<u8>> {
        self.cache.read().ok()?.get(id).cloned()
    }

    /// Store content in cache
    pub fn put(&self, id: TweetId, content: Vec<u8>) {
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(id, content);
        }
    }

    /// Remove content from cache
    pub fn remove(&self, id: &TweetId) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(id);
        }
    }

    /// Clear all cached content
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.read().ok().map(|c| c.len()).unwrap_or(0)
    }
}

impl Default for ContentCache {
    fn default() -> Self {
        Self::new()
    }
}
