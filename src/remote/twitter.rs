//! Twitter API client implementation

use crate::dag::commit::TweetId;
use crate::error::{Result, XFilesError};
use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};

const TWITTER_API_BASE: &str = "https://api.twitter.com/2";

/// Twitter API adapter
pub struct TwitterAdapter {
    client: Client,
    bearer_token: String,
}

impl TwitterAdapter {
    /// Create a new Twitter adapter with Bearer Token authentication
    ///
    /// For Twitter API v2, you need a Bearer Token which can be obtained from:
    /// https://developer.twitter.com/en/portal/dashboard
    ///
    /// # Arguments
    /// * `bearer_token` - Your Twitter API Bearer Token (not API key/secret)
    pub fn new(bearer_token: String) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", bearer_token))
                .expect("Invalid bearer token"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            bearer_token,
        }
    }

    /// Create adapter from API key and secret (legacy OAuth 1.0a - deprecated)
    /// For new applications, use `new()` with Bearer Token instead
    #[deprecated(note = "Use new() with Bearer Token for Twitter API v2")]
    pub fn from_api_keys(api_key: String, api_secret: String) -> Self {
        // For backwards compatibility, treat api_key as bearer_token
        let _ = api_secret; // Unused
        Self::new(api_key)
    }

    /// Get a tweet by ID
    pub async fn get_tweet(&self, id: &TweetId) -> Result<Tweet> {
        let url = format!("{}/tweets/{}", TWITTER_API_BASE, id);

        let response = self
            .client
            .get(&url)
            .query(&[("tweet.fields", "created_at,author_id,in_reply_to_user_id,referenced_tweets")])
            .send()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to fetch tweet: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(XFilesError::TwitterApi(format!(
                "Twitter API error {}: {}",
                status, error_text
            )));
        }

        let api_response: TwitterApiResponse<TweetData> = response
            .json()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to parse response: {}", e)))?;

        let tweet_data = api_response
            .data
            .ok_or_else(|| XFilesError::TwitterApi("No tweet data in response".to_string()))?;

        Ok(Tweet::from(tweet_data))
    }

    /// Get replies to a tweet
    /// Note: This uses Twitter search API which may have limitations
    pub async fn get_replies(&self, id: &TweetId) -> Result<Vec<Tweet>> {
        let url = format!("{}/tweets/search/recent", TWITTER_API_BASE);

        // Search for tweets that are replies to this tweet
        let query = format!("conversation_id:{}", id);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("query", query.as_str()),
                ("tweet.fields", "created_at,author_id,in_reply_to_user_id,referenced_tweets"),
                ("max_results", "100"),
            ])
            .send()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to fetch replies: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(XFilesError::TwitterApi(format!(
                "Twitter API error {}: {}",
                status, error_text
            )));
        }

        let api_response: TwitterApiListResponse<TweetData> = response
            .json()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to parse response: {}", e)))?;

        Ok(api_response
            .data
            .unwrap_or_default()
            .into_iter()
            .map(Tweet::from)
            .collect())
    }

    /// Post a new tweet
    pub async fn post_tweet(&self, content: &str) -> Result<TweetId> {
        let url = format!("{}/tweets", TWITTER_API_BASE);

        let payload = CreateTweetRequest {
            text: content.to_string(),
            reply: None,
        };

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to post tweet: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(XFilesError::TwitterApi(format!(
                "Twitter API error {}: {}",
                status, error_text
            )));
        }

        let api_response: TwitterApiResponse<CreatedTweetData> = response
            .json()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to parse response: {}", e)))?;

        let tweet_id = api_response
            .data
            .ok_or_else(|| XFilesError::TwitterApi("No tweet data in response".to_string()))?
            .id;

        Ok(tweet_id)
    }

    /// Post a reply to a tweet
    pub async fn post_reply(&self, parent_id: &TweetId, content: &str) -> Result<TweetId> {
        let url = format!("{}/tweets", TWITTER_API_BASE);

        let payload = CreateTweetRequest {
            text: content.to_string(),
            reply: Some(ReplySettings {
                in_reply_to_tweet_id: parent_id.clone(),
            }),
        };

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to post reply: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(XFilesError::TwitterApi(format!(
                "Twitter API error {}: {}",
                status, error_text
            )));
        }

        let api_response: TwitterApiResponse<CreatedTweetData> = response
            .json()
            .await
            .map_err(|e| XFilesError::TwitterApi(format!("Failed to parse response: {}", e)))?;

        let tweet_id = api_response
            .data
            .ok_or_else(|| XFilesError::TwitterApi("No tweet data in response".to_string()))?
            .id;

        Ok(tweet_id)
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

// ===== Twitter API v2 Response Types =====

/// Twitter API v2 response wrapper
#[derive(Debug, Deserialize)]
struct TwitterApiResponse<T> {
    data: Option<T>,
}

/// Twitter API v2 list response wrapper
#[derive(Debug, Deserialize)]
struct TwitterApiListResponse<T> {
    data: Option<Vec<T>>,
}

/// Tweet data from Twitter API
#[derive(Debug, Deserialize)]
struct TweetData {
    id: String,
    text: String,
    #[serde(default)]
    author_id: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    referenced_tweets: Option<Vec<ReferencedTweet>>,
}

/// Referenced tweet info
#[derive(Debug, Deserialize)]
struct ReferencedTweet {
    #[serde(rename = "type")]
    ref_type: String,
    id: String,
}

impl From<TweetData> for Tweet {
    fn from(data: TweetData) -> Self {
        // Extract in_reply_to from referenced_tweets
        let in_reply_to = data
            .referenced_tweets
            .and_then(|refs| {
                refs.into_iter()
                    .find(|r| r.ref_type == "replied_to")
                    .map(|r| r.id)
            });

        Tweet {
            id: data.id,
            author_id: data.author_id.unwrap_or_default(),
            text: data.text,
            created_at: data.created_at.unwrap_or_default(),
            in_reply_to,
        }
    }
}

/// Created tweet response data
#[derive(Debug, Deserialize)]
struct CreatedTweetData {
    id: String,
}

/// Request to create a tweet
#[derive(Debug, Serialize)]
struct CreateTweetRequest {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<ReplySettings>,
}

/// Reply settings for creating a reply tweet
#[derive(Debug, Serialize)]
struct ReplySettings {
    in_reply_to_tweet_id: String,
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
