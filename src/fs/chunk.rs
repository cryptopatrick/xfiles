//! Content chunking for tweets (280 char limit)

use crate::error::Result;

/// Maximum size for a single tweet (in bytes)
pub const TWEET_MAX_SIZE: usize = 280;

/// Split content into tweet-sized chunks
pub fn chunk_content(content: &[u8]) -> Result<Vec<Vec<u8>>> {
    if content.len() <= TWEET_MAX_SIZE {
        return Ok(vec![content.to_vec()]);
    }

    let mut chunks = Vec::new();
    let mut offset = 0;

    while offset < content.len() {
        let end = (offset + TWEET_MAX_SIZE).min(content.len());
        chunks.push(content[offset..end].to_vec());
        offset = end;
    }

    Ok(chunks)
}

/// Recombine chunks into original content
pub fn recombine_chunks(chunks: &[Vec<u8>]) -> Result<Vec<u8>> {
    Ok(chunks.concat())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_small_content() {
        let content = b"Hello, world!";
        let chunks = chunk_content(content).unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], content);
    }

    #[test]
    fn test_chunk_large_content() {
        let content = vec![b'x'; 500];
        let chunks = chunk_content(&content).unwrap();
        assert!(chunks.len() > 1);

        let recombined = recombine_chunks(&chunks).unwrap();
        assert_eq!(recombined, content);
    }
}
