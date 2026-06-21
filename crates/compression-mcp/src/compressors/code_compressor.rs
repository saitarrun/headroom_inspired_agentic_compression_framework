use super::Compressor;
use mcp_types::MpcError;

/// CodeCompressor: Code-specific compression.
/// Preserves signal elements (function signatures, line numbers, error messages)
/// while removing noise (timestamps, retry counts, formatting).
///
/// TODO: Implement code signal map, stack trace parsing, diff compression.
pub struct CodeCompressor;

impl Compressor for CodeCompressor {
    fn compress(&self, content: &str) -> Result<(String, f64), MpcError> {
        // Stub implementation: return input unchanged
        let ratio = 1.0;
        Ok((content.to_string(), ratio))
    }

    fn name(&self) -> &str {
        "CodeCompressor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_compressor_returns_output() {
        let compressor = CodeCompressor;
        let input = r#"at function () (file.rs:42:10)
  at handler () (main.rs:100:5)"#;
        let (output, ratio) = compressor.compress(input).expect("compress failed");
        assert_eq!(output, input);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_code_compressor_name() {
        let compressor = CodeCompressor;
        assert_eq!(compressor.name(), "CodeCompressor");
    }
}
