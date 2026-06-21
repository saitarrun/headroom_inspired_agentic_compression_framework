/// Safety invariants for compression.
/// These rules ensure that critical information is never compressed.

const SAFETY_KEYWORDS: &[&str] = &[
    "authorization",
    "auth",
    "bearer",
    "token",
    "api_key",
    "secret",
    "password",
    "error:",
    "exception:",
    "fatal",
    "panic",
];

/// Check if content contains critical safety information that should not be compressed.
pub fn should_skip_compression(content: &str) -> bool {
    let lower = content.to_lowercase();
    SAFETY_KEYWORDS.iter().any(|keyword| lower.contains(keyword))
}

/// Extract critical information that must be preserved.
pub fn extract_critical_info(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("error") || lower.contains("fatal") || lower.contains("exception")
        })
        .map(|s| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_compression_for_auth_headers() {
        assert!(should_skip_compression("Authorization: Bearer token123"));
    }

    #[test]
    fn test_should_skip_compression_for_api_keys() {
        assert!(should_skip_compression("api_key: sk-1234567890"));
    }

    #[test]
    fn test_should_skip_compression_for_errors() {
        assert!(should_skip_compression("error: connection timeout"));
    }

    #[test]
    fn test_should_not_skip_compression_for_regular_output() {
        assert!(!should_skip_compression("Response: success"));
    }

    #[test]
    fn test_extract_critical_info() {
        let content = "line 1\nError: something failed\nline 3\nFatal: system down\n";
        let critical = extract_critical_info(content);
        assert_eq!(critical.len(), 2);
        assert!(critical[0].contains("Error"));
        assert!(critical[1].contains("Fatal"));
    }
}
