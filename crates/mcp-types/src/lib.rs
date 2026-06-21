use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MpcError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Unknown tool: {0}")]
    UnknownTool(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressRequest {
    pub tool_name: String,
    pub raw_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressResponse {
    pub output_id: String,
    pub compressed_output: String,
    pub compression_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveRequest {
    pub output_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveResponse {
    pub original_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsRequest {
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub tokens_saved: u64,
    pub accuracy_delta: f64,
    pub workload_reduction: f64,
}

/// Content type for routing compression algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Code,
    Text,
    Unknown,
}

impl ContentType {
    pub fn detect(content: &str) -> Self {
        let trimmed = content.trim();

        // Try JSON
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return ContentType::Json;
        }

        // Try code (stack traces, diffs, function signatures)
        if trimmed.contains("at ") || trimmed.contains("File \"")
            || trimmed.contains("line ") || trimmed.contains("---")
            || trimmed.contains("fn ") || trimmed.contains("pub fn ")
            || trimmed.contains("def ") {
            return ContentType::Code;
        }

        // Default to text
        ContentType::Text
    }
}
