<h1 align="center">
  <br>
    <img
      src="https://github.com/cryptopatrick/factory/blob/master/img/100days/xfiles.png"
      alt="xfiles"
      width="200"
    />
  <br>
  XFILES
  <br>
</h1>

<h4 align="center">
  Twitter (X) as a public filesystem for AI agents.
</h4>

<p align="center">
  <a href="https://crates.io/crates/xfiles" target="_blank">
    <img src="https://img.shields.io/crates/v/xfiles" alt="Crates.io"/>
  </a>
  <a href="https://crates.io/crates/xfiles" target="_blank">
    <img src="https://img.shields.io/crates/d/xfiles" alt="Downloads"/>
  </a>
  <a href="https://docs.rs/xfiles" target="_blank">
    <img src="https://docs.rs/xfiles/badge.svg" alt="Documentation"/>
  </a>
  <a href="LICENSE" target="_blank">
    <img src="https://img.shields.io/github/license/sulu/sulu.svg" alt="GitHub license"/>
  </a>
</p>

<b>Author's bio:</b> 👋😀 Hi, I'm CryptoPatrick! I'm currently enrolled as an
Undergraduate student in Mathematics, at Chalmers & the University of Gothenburg, Sweden. <br>
If you have any questions or need more info, then please <a href="https://discord.gg/T8EWmJZpCB">join my Discord Channel: AiMath</a>

---

<p align="center">
  <a href="#-what-is-xfiles">What is xfiles</a> •
  <a href="#-features">Features</a> •
  <a href="#-how-to-use">How To Use</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-license">License</a>
</p>

## 🛎 Important Notices
* **Twitter API Required**: Requires Twitter API v2 Bearer Token for production use
* **Storage**: Uses Twitter threads as remote storage, SQLite for local indexing
* **Experimental**: v0.1 is a proof-of-concept suitable for research and creative projects
* **Public by Default**: All data is visible on Twitter (encryption coming in v0.2)

<!-- TABLE OF CONTENTS -->
<h2 id="table-of-contents"> :pushpin: Table of Contents</h2>

<details open="open">
  <summary>Table of Contents</summary>
  <ol>
    <li><a href="#-what-is-xfiles">What is xfiles</a></li>
    <li><a href="#-features">Features</a></li>
      <ul>
        <li><a href="#-core-functionality">Core Functionality</a></li>
        <li><a href="#-filesystem-operations">Filesystem Operations</a></li>
        <li><a href="#-twitter-integration">Twitter Integration</a></li>
        <li><a href="#-persistence">Persistence</a></li>
      </ul>
    <li><a href="#-architecture">Architecture</a></li>
    <li><a href="#-how-to-use">How to Use</a></li>
    <li><a href="#-examples">Examples</a></li>
    <li><a href="#-testing">Testing</a></li>
    <li><a href="#-documentation">Documentation</a></li>
    <li><a href="#-license">License</a>
  </ol>
</details>

## 🤔 What is xfiles

`xfiles` is a Rust library that treats Twitter as a public, append-only, log-structured filesystem. Tweets become "files", replies become "commits", and a local SQLite index keeps traversal fast.

**Why?** For transparent AI agents, public verifiability, distributed state, and creative experiments where Twitter serves as a globally verifiable shared memory bus.

### Core Concept

```
Tweet (root)  →  File
Reply         →  Commit
Thread        →  Version history
SQLite        →  Local index/cache
```

### Use Cases

- **AI Agent Memory**: Agents persist state to Twitter for transparency and recovery
- **Multi-Agent Collaboration**: Agents coordinate through shared Twitter threads
- **Public Audit Trails**: All operations are publicly visible and timestamped
- **Distributed State**: No single party controls the substrate
- **Creative Experiments**: Explore novel uses of social platforms as infrastructure

## 📷 Features

`xfiles` provides a complete filesystem abstraction over Twitter with persistent local caching:

### 🔧 Core Functionality
- **Tweet as File Root**: Each file starts with a root tweet
- **Reply as Commit**: Updates are posted as replies, forming a version chain
- **Append-Only DAG**: Git-like directed acyclic graph for version history
- **SQLite Indexing**: Fast local queries without hitting Twitter API

### 📂 Filesystem Operations
- **File Creation**: `open(path, Create)` posts a root tweet
- **Reading**: `read()` fetches content from Twitter (cached locally)
- **Writing**: `write(content)` posts reply commits
- **History**: `history(path)` retrieves full commit chain
- **Listing**: `list(dir)` shows all files in a directory
- **Existence Checks**: `exists(path)` queries local index

### 🐦 Twitter Integration
- **Twitter API v2**: Full integration with modern Twitter API
- **Bearer Token Auth**: Simple authentication with Bearer Tokens
- **Rate Limiting**: Automatic backoff and retry logic
- **Chunking**: Transparent splitting of content >280 characters
- **Error Handling**: Robust error handling for API failures

### 💾 Persistence
- **SQLite Storage**: Reliable file-based persistence
- **Commit Tracking**: DAG of all commits with timestamps
- **Path Mapping**: Files map to Twitter thread roots
- **Content Caching**: Avoid redundant API calls
- **Session Continuity**: Resume operations across restarts

## 📐 Architecture

1. 🏛 Overall Architecture
```diagram
┌──────────────────────────────────────────────────────────┐
│           User Application (Agent/CLI/Backend)           │
│                Single call: fs.open().write()            │
└──────────────────────┬───────────────────────────────────┘
                       │
┌──────────────────────▼───────────────────────────────────┐
│                      XFS Component                       │
│  • Open files (create root tweets)                      │
│  • Read content (fetch from Twitter)                    │
│  • Write updates (post reply tweets)                    │
│  • List files (query SQLite index)                      │
│  • Track history (traverse DAG)                         │
└──────────────┬──────────────────────────┬────────────────┘
               │                          │
       ┌───────▼────────┐        ┌────────▼─────────┐
       │ Twitter Adapter│        │  SQLite Store    │
       │  • API calls   │        │  • Commit index  │
       │  • Rate limit  │        │  • File mapping  │
       │  • Chunking    │        │  • Cache layer   │
       └────────┬───────┘        └──────────────────┘
                │
        ┌───────▼────────┐
        │  Twitter API   │
        │  • GET tweet   │
        │  • POST tweet  │
        │  • GET replies │
        └────────────────┘
```

User → XFS → SQLite Index + Twitter API → Remote Storage

2. 🚃 Data Flow Diagram

```diagram
┌──────────────────────────────────────────────────────────┐
│             file.write("Agent state v2")                 │
└──────────────────────┬───────────────────────────────────┘
                       │
              ┌────────▼────────┐
              │  1. Compute     │
              │     Hash        │
              │  (blake3)       │
              └─────────┬───────┘
                        │
                        │ content hash
                        ▼
              ┌────────────────────┐
              │  2. Chunk          │
              │     Content        │
              │  (if >280 chars)   │
              └─────────┬──────────┘
                        │
                        │ chunks[]
                        ▼
              ┌────────────────────┐
              │  3. Post Reply     │
              │     to Twitter     │
              │  (TwitterAdapter)  │
              └─────────┬──────────┘
                        │
                        │ tweet_id
                        ▼
              ┌────────────────────┐
              │  4. Create Commit  │
              │  • id = tweet_id   │
              │  • parent = head   │
              │  • hash, timestamp │
              └─────────┬──────────┘
                        │
                        ▼
              ┌────────────────────┐
              │  5. Store Commit   │
              │     in SQLite      │
              │  • Update index    │
              │  • Mark as head    │
              │  • Cache content   │
              └────────────────────┘
```

3. 💾 Storage Layer Architecture

```diagram
┌──────────────────────────────────────────────────────────┐
│                          XFS                             │
│  ┌─────────────────────────────────────────────────────┐ │
│  │                   Public API                        │ │
│  │  • open(path, mode) → XFile                         │ │
│  │  • list(dir) → Vec<String>                          │ │
│  │  • history(path) → Vec<Commit>                      │ │
│  │  • exists(path) → bool                              │ │
│  └─────────────────────────────────────────────────────┘ │
│                               │                          │
│  ┌────────────────────────────▼─────────────────────────┐│
│  │               XFile Operations                       ││
│  │  • read() → Vec<u8>                                  ││
│  │  • write(content) → Result<()>                       ││
│  │  • delete() → Result<()>                             ││
│  └────────────────────────────┬─────────────────────────┘│
└───────────────────────────────┬───────────────────────────┘
                                │
              ┌─────────────────▼────────────────────┐
              │        SQLite Database (file.db)     │
              │  ┌─────────────────────────────────┐ │
              │  │            files                │ │
              │  │  - path (PK)                    │ │
              │  │  - root_tweet_id                │ │
              │  │  - created_at                   │ │
              │  └─────────────────────────────────┘ │
              │                                      │
              │  ┌─────────────────────────────────┐ │
              │  │           commits               │ │
              │  │  - tweet_id (PK)                │ │
              │  │  - parent_id (JSON array)       │ │
              │  │  - timestamp                    │ │
              │  │  - author                       │ │
              │  │  - hash (blake3)                │ │
              │  │  - mime                         │ │
              │  │  - size                         │ │
              │  │  - head (boolean)               │ │
              │  └─────────────────────────────────┘ │
              │                                      │
              │  ┌─────────────────────────────────┐ │
              │  │           chunks                │ │
              │  │  - tweet_id (PK)                │ │
              │  │  - parent_commit (FK)           │ │
              │  │  - idx (chunk order)            │ │
              │  │  - size, hash                   │ │
              │  └─────────────────────────────────┘ │
              └──────────────────────────────────────┘
```

4. ⏳ Commit Lifecycle

```diagram
┌──────────────────────────────────────────────────────┐
│            User calls file.write(content)            │
└───────────────────┬──────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                  1. Chunk Content                      │
│  • Split into 280-byte chunks if needed                │
│  • Compute blake3 hash of full content                 │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              2. Post to Twitter API                    │
│  • Post first chunk as reply to current head           │
│  • Post remaining chunks as reply chain                │
│  • Receive tweet_id for each chunk                     │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│                 3. Create Commit Object                │
│  • id = first chunk tweet_id                           │
│  • parents = [current_head]                            │
│  • hash = content hash                                 │
│  • timestamp = now()                                   │
│  • author = username                                   │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              4. Persist to SQLite                      │
│  • INSERT INTO commits                                 │
│  • UPDATE files SET head                               │
│  • Cache content for fast reads                        │
└───────────────────┬────────────────────────────────────┘
                    │
                    ▼
┌────────────────────────────────────────────────────────┐
│              5. Publicly Visible on Twitter            │
│  • Tweet URL: https://twitter.com/i/web/status/{id}   │
│  • Timestamped by Twitter                              │
│  • Immutable and auditable                             │
└────────────────────────────────────────────────────────┘
```

## 🚙 How to Use

### Installation

Add `xfiles` to your `Cargo.toml`:

```toml
[dependencies]
xfiles = "0.1"
tokio = { version = "1", features = ["full"] }
```

Or install with cargo:

```bash
cargo add xfiles
```

### Twitter API Setup

Before using xfiles with real Twitter, you need API credentials:

1. Go to https://developer.twitter.com/en/portal/dashboard
2. Create a project and app
3. Generate Bearer Token under "Keys and tokens"
4. See [docs/TWITTER_SETUP.md](docs/TWITTER_SETUP.md) for detailed instructions

```bash
export TWITTER_BEARER_TOKEN="your_bearer_token_here"
export TWITTER_USERNAME="your_username"
```

### Basic Example (Mock Adapter)

```rust
use xfiles::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Use mock adapter for testing (no Twitter API needed)
    let adapter = Arc::new(MockAdapter::new());
    let mut fs = XFS::with_adapter("agent", adapter, Some(":memory:")).await?;

    // Create and write to file
    let mut file = fs.open("memory.txt", OpenMode::Create).await?;
    file.write(b"Agent state v1").await?;

    // Read it back
    let content = file.read().await?;
    println!("{}", String::from_utf8_lossy(&content));

    // Multiple writes create commit chain
    file.write(b"Agent state v2").await?;
    file.write(b"Agent state v3").await?;

    // Get history
    let history = fs.history("memory.txt").await?;
    println!("Total commits: {}", history.len());

    Ok(())
}
```

### Real Twitter API Example

```rust
use xfiles::*;

#[tokio::main]
async fn main() -> Result<()> {
    let bearer_token = std::env::var("TWITTER_BEARER_TOKEN")?;

    // Connect with real Twitter API
    let mut fs = XFS::connect("@myagent", &bearer_token).await?;

    // Create file (posts root tweet)
    let mut file = fs.open("agent_memory.txt", OpenMode::Create).await?;

    // Write content (posts reply tweet)
    file.write(b"Day 1: Agent initialized").await?;

    // Read it back (fetches from Twitter)
    let content = file.read().await?;

    println!("Content: {}", String::from_utf8_lossy(&content));
    println!("Tweet URL: https://twitter.com/i/web/status/{}", file.head());

    Ok(())
}
```

### Advanced Usage

```rust
use xfiles::*;

#[tokio::main]
async fn main() -> Result<()> {
    let bearer_token = std::env::var("TWITTER_BEARER_TOKEN")?;
    let mut fs = XFS::connect("@myagent", &bearer_token).await?;

    // Create multiple files
    let mut memory = fs.open("agent/memory.json", OpenMode::Create).await?;
    let mut log = fs.open("agent/debug.log", OpenMode::Create).await?;

    // Write to different files
    memory.write(br#"{"state": "active", "version": 1}"#).await?;
    log.write(b"[INFO] Agent started").await?;

    // List all files
    let files = fs.list("agent").await?;
    println!("Files in agent/: {:?}", files);

    // Get full history of a file
    let history = fs.history("agent/memory.json").await?;
    for (i, commit) in history.iter().enumerate() {
        println!("Commit {}: {} at {}",
            i + 1,
            commit.id,
            commit.timestamp
        );
    }

    // Check file existence
    if fs.exists("agent/config.toml").await? {
        println!("Config file exists");
    }

    Ok(())
}
```

## 🧪 Examples

The repository includes several examples demonstrating different features:

```bash
# Mock adapter example (no Twitter API needed)
cargo run --example basic

# Real Twitter API example (requires credentials)
export TWITTER_BEARER_TOKEN="your_token"
cargo run --example twitter_real
```

### Example Output

```
=== xfiles Basic Example ===

1. Creating a new file...
   ✓ Created file: memory.txt

2. Writing to file...
   ✓ Wrote initial content

3. Reading file content...
   Content: Day 1: Agent initialized

4. Writing multiple updates...
   ✓ Created commit chain

5. Reading latest version...
   Latest: Day 3: Successfully stored memory

6. Getting file history...
   Total commits: 4
   Commit 1: mock_tweet_1 (2026-01-16 12:28:15 UTC)
   Commit 2: mock_tweet_2 (2026-01-16 12:28:15 UTC)
   Commit 3: mock_tweet_3 (2026-01-16 12:28:15 UTC)
   Commit 4: mock_tweet_4 (2026-01-16 12:28:15 UTC)
```

## 🧪 Testing

Run the test suite:

```bash
# Run all tests (uses mock adapter)
cargo test

# Run specific test suite
cargo test --test tests

# Run with output
cargo test -- --nocapture
```

All tests use the MockAdapter, so no Twitter API credentials are needed for testing.

## 📚 Documentation

Comprehensive documentation is available:

- **[API Documentation](https://docs.rs/xfiles)** - Full API reference
- **[Twitter Setup Guide](docs/TWITTER_SETUP.md)** - Get Twitter API credentials and setup
- **[Contributing Guide](CONTRIBUTING.md)** - Development guidelines and workflow

### Key Concepts

- **Files**: Mapped to Twitter thread roots, tracked in SQLite
- **Commits**: Each write creates a new tweet reply
- **DAG**: Git-like directed acyclic graph for version history
- **Chunking**: Content >280 chars automatically split across multiple tweets
- **Caching**: SQLite caches content to minimize API calls

### Performance Notes

- **Reads**: Cached in SQLite (no API call on cache hit)
- **Writes**: Rate-limited by Twitter API (300 tweets/15min)
- **Chunking**: Transparent for content > 280 chars
- **Rate Limiting**: Automatic exponential backoff

Twitter API v2 Free Tier limits:
- 50 tweet reads / 15 min
- 300 tweet posts / 15 min

## 🖊 Author

<a href="https://x.com/cryptopatrick">CryptoPatrick</a>

Keybase Verification:
https://keybase.io/cryptopatrick/sigs/8epNh5h2FtIX1UNNmf8YQ-k33M8J-Md4LnAN

## 🐣 Support
Leave a ⭐ if you think this project is cool.

## 🗄 License

This project is licensed under MIT. See [LICENSE](LICENSE) for details.

---

**Inspired by:**
- Git (DAG commits, history)
- IPFS (content-addressed chunks)
- CRDTs (distributed updates)
- Blockchains (timestamped logs)

**Made with ☕ for transparent AI agents**
