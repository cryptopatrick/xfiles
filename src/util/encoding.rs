//! Content encoding and compression utilities

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Metadata header for encoded content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentHeader {
    /// MIME type
    pub mime: String,
    /// Original size
    pub size: usize,
    /// Content hash
    pub hash: String,
    /// Whether content is compressed
    pub compressed: bool,
    /// Encoding version
    pub version: u8,
}

/// Encode content with metadata header
pub fn encode_with_header(content: &[u8], mime: &str) -> Result<Vec<u8>> {
    let hash = crate::util::hash::compute_hash(content);

    let header = ContentHeader {
        mime: mime.to_string(),
        size: content.len(),
        hash,
        compressed: false,
        version: 1,
    };

    let header_json = serde_json::to_string(&header)?;
    let separator = b"\n---\n";

    let mut encoded = Vec::new();
    encoded.extend_from_slice(header_json.as_bytes());
    encoded.extend_from_slice(separator);
    encoded.extend_from_slice(content);

    Ok(encoded)
}

/// Decode content and extract header
pub fn decode_with_header(encoded: &[u8]) -> Result<(ContentHeader, Vec<u8>)> {
    let separator = b"\n---\n";

    if let Some(pos) = encoded
        .windows(separator.len())
        .position(|window| window == separator)
    {
        let header_bytes = &encoded[..pos];
        let content = &encoded[pos + separator.len()..];

        let header: ContentHeader = serde_json::from_slice(header_bytes)?;

        Ok((header, content.to_vec()))
    } else {
        Err(crate::error::XFilesError::InvalidEncoding(
            "Missing header separator".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let content = b"test content";
        let encoded = encode_with_header(content, "text/plain").unwrap();
        let (header, decoded) = decode_with_header(&encoded).unwrap();

        assert_eq!(header.mime, "text/plain");
        assert_eq!(header.size, content.len());
        assert_eq!(decoded, content);
    }
}
