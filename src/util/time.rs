//! Time and timestamp utilities

use chrono::{DateTime, Utc};

/// Get current UTC timestamp
pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Parse timestamp from string
pub fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    s.parse()
}

/// Format timestamp as ISO 8601
pub fn format_timestamp(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now() {
        let ts = now();
        assert!(ts.timestamp() > 0);
    }

    #[test]
    fn test_format_parse_roundtrip() {
        let original = now();
        let formatted = format_timestamp(&original);
        let parsed = parse_timestamp(&formatted).unwrap();

        // Compare timestamps (subsecond precision may differ slightly)
        assert_eq!(original.timestamp(), parsed.timestamp());
    }
}
