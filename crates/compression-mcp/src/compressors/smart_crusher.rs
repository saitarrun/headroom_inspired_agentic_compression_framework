use super::Compressor;
use mcp_types::MpcError;

/// SmartCrusher: JSON-specific compression.
/// Preserves signal fields (keys, values, structure) while removing noise (whitespace, metadata).
///
/// TODO: Implement JSON signal map and field detection.
pub struct SmartCrusher;

impl Compressor for SmartCrusher {
    fn compress(&self, content: &str) -> Result<(String, f64), MpcError> {
        // Stub implementation: return input unchanged
        let original_len = content.len() as f64;
        let ratio = 1.0;
        Ok((content.to_string(), ratio))
    }

    fn name(&self) -> &str {
        "SmartCrusher"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_crusher_returns_output() {
        let crusher = SmartCrusher;
        let input = r#"{"status": "ok", "data": [1, 2, 3]}"#;
        let (output, ratio) = crusher.compress(input).expect("compress failed");
        assert_eq!(output, input);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_smart_crusher_name() {
        let crusher = SmartCrusher;
        assert_eq!(crusher.name(), "SmartCrusher");
    }
}
