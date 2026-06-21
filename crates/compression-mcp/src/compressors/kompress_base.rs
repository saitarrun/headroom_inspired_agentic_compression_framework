use super::Compressor;
use mcp_types::MpcError;

/// KompressBase: Text/prose compression.
/// Uses a language model (local or cloud-based) to compress general text
/// while preserving error messages and diagnostic information.
///
/// TODO: Implement model loading (ONNX or cloud API), inference pipeline.
pub struct KompressBase;

impl Compressor for KompressBase {
    fn compress(&self, content: &str) -> Result<(String, f64), MpcError> {
        // Stub implementation: return input unchanged
        let ratio = 1.0;
        Ok((content.to_string(), ratio))
    }

    fn name(&self) -> &str {
        "KompressBase"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kompress_base_returns_output() {
        let compressor = KompressBase;
        let input = "This is a test log message with some noise and metadata";
        let (output, ratio) = compressor.compress(input).expect("compress failed");
        assert_eq!(output, input);
        assert_eq!(ratio, 1.0);
    }

    #[test]
    fn test_kompress_base_name() {
        let compressor = KompressBase;
        assert_eq!(compressor.name(), "KompressBase");
    }
}
