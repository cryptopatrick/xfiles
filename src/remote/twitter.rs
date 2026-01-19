//! Twitter API client implementation with OAuth 1.0a

use crate::dag::commit::TweetId;
use crate::error::{Result, XFilesError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use oauth::{Token, HmacSha1};

const TWITTER_API_BASE: &str = "https://api.twitter.com/2";

/// Twitter API adapter with OAuth 1.0a authentication
pub struct TwitterAdapter {
    client: Client,
    token: Token<Box<str>>,
}

impl TwitterAdapter {
    /// Create a new Twitter adapter with OAuth 1.0a authentication
    ///
    /// For Twitter API v2 write operations, you need OAuth 1.0a credentials:
    /// - Consumer Key (API Key)
    /// - Consumer Secret (API Secret)
    /// - Access Token
    /// - Access Token Secret
    ///
    /// Get these from: https://developer.twitter.com/en/portal/dashboard
    ///
    /// # Arguments
    /// * `consumer_key` - Your Twitter API Key
    /// * `consumer_secret` - Your Twitter API Secret
    /// * `access_token` - Your Access Token
    /// * `access_token_secret` - Your Access Token Secret
    pub fn new(
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Self {
        let client = Client::builder()
            .build()
            .expect("Failed to build HTTP client");

        let token = Token::from_parts(
            consumer_key.into(),
            consumer_secret.into(),
            access_token.into(),
            access_token_secret.into(),
        );

        Self { client, token }
    }

    /// Generate OAuth 1.0a Authorization header
    fn generate_oauth_header(&self, method: &str, url: &str) -> String {
        if method == "POST" {
            oauth::post(url, &(), &self.token, HmacSha1)
        } else {
            oauth::get(url, &(), &self.token, HmacSha1)
        }
    }

    /// Get a tweet by ID
    pub async fn get_tweet(&self, id: &TweetId) -> Result<Tweet> {
        let base_url = format!("{}/tweets/{}", TWITTER_API_BASE, id);
        let url_with_params = format!("{}?tweet.fields=created_at,author_id,in_reply_to_user_id,referenced_tweets", base_url);

        let auth_header = self.generate_oauth_header("GET", &url_with_params);

        let response = self
            .client
            .get(&base_url)
            .query(&[("tweet.fields", "created_at,author_id,in_reply_to_user_id,referenced_tweets")])
            .header("Authorization", auth_header)
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
    pub async fn get_replies(&self, id: &TweetId) -> Result<Vec<Tweet>> {
        let base_url = format!("{}/tweets/search/recent", TWITTER_API_BASE);
        let query = format!("conversation_id:{}", id);
        // Note: OAuth library will handle URL encoding
        let url_with_params = format!(
            "{}?query={}&tweet.fields=created_at,author_id,in_reply_to_user_id,referenced_tweets&max_results=100",
            base_url, query
        );

        let auth_header = self.generate_oauth_header("GET", &url_with_params);

        let response = self
            .client
            .get(&base_url)
            .query(&[
                ("query", query.as_str()),
                ("tweet.fields", "created_at,author_id,in_reply_to_user_id,referenced_tweets"),
                ("max_results", "100"),
            ])
            .header("Authorization", auth_header)
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

        let auth_header = self.generate_oauth_header("POST", &url);

        let payload = CreateTweetRequest {
            text: content.to_string(),
            reply: None,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
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

        let auth_header = self.generate_oauth_header("POST", &url);

        let payload = CreateTweetRequest {
            text: content.to_string(),
            reply: Some(ReplySettings {
                in_reply_to_tweet_id: parent_id.clone(),
            }),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
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
