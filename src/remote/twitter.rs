//! Twitter API client implementation

use crate::dag::commit::TweetId;
use crate::error::Result;
use async_trait::async_trait;
use reqwest::Client;

/// Twitter API adapter
pub struct TwitterAdapter {
    client: Client,
    api_key: String,
    api_secret: String,
}

impl TwitterAdapter {
    /// Create a new Twitter adapter
    pub fn new(api_key: String, api_secret: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            api_secret,
        }
    }

    /// Get a tweet by ID
    pub async fn get_tweet(&self, _id: &TweetId) -> Result<Tweet> {
        todo!("Implement tweet fetch")
    }

    /// Get replies to a tweet
    pub async fn get_replies(&self, _id: &TweetId) -> Result<Vec<Tweet>> {
        todo!("Implement reply fetch")
    }

    /// Post a new tweet
    pub async fn post_tweet(&self, _content: &str) -> Result<TweetId> {
        todo!("Implement tweet posting")
    }

    /// Post a reply to a tweet
    pub async fn post_reply(&self, _parent_id: &TweetId, _content: &str) -> Result<TweetId> {
        todo!("Implement reply posting")
    }
}

/// Represents a tweet from the API
#[derive(Debug, Clone)]
pub struct Tweet {
    pub id: TweetId,
    pub author_id: String,
    pub text: String,
    pub created_at: String,
    pub in_reply_to: Option<TweetId>,
}

/// Trait for remote storage adapters (allows multiple backends)
#[async_trait]
pub trait RemoteAdapter: Send + Sync {
    /// Fetch content by ID
    async fn fetch(&self, id: &TweetId) -> Result<Vec<u8>>;

    /// Store content and return ID
    async fn store(&self, content: &[u8]) -> Result<TweetId>;

    /// Store content as reply to parent
    async fn store_reply(&self, parent_id: &TweetId, content: &[u8]) -> Result<TweetId>;

    /// Fetch all replies to a tweet
    async fn fetch_replies(&self, id: &TweetId) -> Result<Vec<TweetId>>;
}

#[async_trait]
impl RemoteAdapter for TwitterAdapter {
    async fn fetch(&self, id: &TweetId) -> Result<Vec<u8>> {
        let tweet = self.get_tweet(id).await?;
        Ok(tweet.text.into_bytes())
    }

    async fn store(&self, content: &[u8]) -> Result<TweetId> {
        let text = String::from_utf8_lossy(content);
        self.post_tweet(&text).await
    }

    async fn store_reply(&self, parent_id: &TweetId, content: &[u8]) -> Result<TweetId> {
        let text = String::from_utf8_lossy(content);
        self.post_reply(parent_id, &text).await
    }

    async fn fetch_replies(&self, id: &TweetId) -> Result<Vec<TweetId>> {
        let replies = self.get_replies(id).await?;
        Ok(replies.into_iter().map(|t| t.id).collect())
    }
}
