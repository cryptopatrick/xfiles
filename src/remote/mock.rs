//! Mock adapter for testing without real Twitter API

use crate::dag::commit::TweetId;
use crate::error::Result;
use crate::remote::twitter::{RemoteAdapter, Tweet};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock adapter that simulates Twitter API in memory
#[derive(Clone)]
pub struct MockAdapter {
    /// In-memory tweet storage
    tweets: Arc<Mutex<HashMap<TweetId, MockTweet>>>,
    /// Counter for generating tweet IDs
    next_id: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone)]
struct MockTweet {
    id: TweetId,
    content: Vec<u8>,
    parent_id: Option<TweetId>,
    author: String,
}

impl MockAdapter {
    /// Create a new mock adapter
    pub fn new() -> Self {
        Self {
            tweets: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Generate a new tweet ID
    fn generate_id(&self) -> TweetId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = format!("mock_tweet_{}", *next_id);
        *next_id += 1;
        id
    }

    /// Get a tweet by ID
    pub fn get_tweet(&self, id: &TweetId) -> Option<Tweet> {
        let tweets = self.tweets.lock().unwrap();
        tweets.get(id).map(|t| Tweet {
            id: t.id.clone(),
            author_id: t.author.clone(),
            text: String::from_utf8_lossy(&t.content).to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            in_reply_to: t.parent_id.clone(),
        })
    }

    /// Get all replies to a tweet
    pub fn get_replies(&self, id: &TweetId) -> Vec<TweetId> {
        let tweets = self.tweets.lock().unwrap();
        tweets
            .values()
            .filter(|t| t.parent_id.as_ref() == Some(id))
            .map(|t| t.id.clone())
            .collect()
    }
}

impl Default for MockAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RemoteAdapter for MockAdapter {
    async fn fetch(&self, id: &TweetId) -> Result<Vec<u8>> {
        let tweets = self.tweets.lock().unwrap();
        tweets
            .get(id)
            .map(|t| t.content.clone())
            .ok_or_else(|| {
                crate::error::XFilesError::TwitterApi(format!("Tweet not found: {}", id))
            })
    }

    async fn store(&self, content: &[u8]) -> Result<TweetId> {
        let id = self.generate_id();
        let tweet = MockTweet {
            id: id.clone(),
            content: content.to_vec(),
            parent_id: None,
            author: "mock_user".to_string(),
        };

        let mut tweets = self.tweets.lock().unwrap();
        tweets.insert(id.clone(), tweet);

        Ok(id)
    }

    async fn store_reply(&self, parent_id: &TweetId, content: &[u8]) -> Result<TweetId> {
        let id = self.generate_id();
        let tweet = MockTweet {
            id: id.clone(),
            content: content.to_vec(),
            parent_id: Some(parent_id.clone()),
            author: "mock_user".to_string(),
        };

        let mut tweets = self.tweets.lock().unwrap();
        tweets.insert(id.clone(), tweet);

        Ok(id)
    }

    async fn fetch_replies(&self, id: &TweetId) -> Result<Vec<TweetId>> {
        Ok(self.get_replies(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_adapter_store_fetch() {
        let adapter = MockAdapter::new();
        let content = b"Hello, world!";

        let id = adapter.store(content).await.unwrap();
        let fetched = adapter.fetch(&id).await.unwrap();

        assert_eq!(fetched, content);
    }

    #[tokio::test]
    async fn test_mock_adapter_replies() {
        let adapter = MockAdapter::new();

        let root_id = adapter.store(b"Root tweet").await.unwrap();
        let reply1_id = adapter.store_reply(&root_id, b"Reply 1").await.unwrap();
        let reply2_id = adapter.store_reply(&root_id, b"Reply 2").await.unwrap();

        let replies = adapter.fetch_replies(&root_id).await.unwrap();

        assert_eq!(replies.len(), 2);
        assert!(replies.contains(&reply1_id));
        assert!(replies.contains(&reply2_id));
    }
}
