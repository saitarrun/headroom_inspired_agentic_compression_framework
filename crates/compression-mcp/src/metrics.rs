use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::collections::HashMap;

/// Metrics collector for compression operations.
pub struct MetricsCollector {
    total_tokens_saved: Arc<AtomicU64>,
    compressions_count: Arc<AtomicU64>,
    errors_count: Arc<AtomicU64>,
    accuracy_scores: Arc<Mutex<Vec<f64>>>,
}

impl MetricsCollector {
    /// Create a new metrics collector.
    pub fn new() -> Self {
        Self {
            total_tokens_saved: Arc::new(AtomicU64::new(0)),
            compressions_count: Arc::new(AtomicU64::new(0)),
            errors_count: Arc::new(AtomicU64::new(0)),
            accuracy_scores: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a successful compression.
    pub fn record_compression(&self, tokens_saved: u64) {
        self.total_tokens_saved.fetch_add(tokens_saved, Ordering::Relaxed);
        self.compressions_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a compression error.
    pub fn record_error(&self) {
        self.errors_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an accuracy score (0.0 - 1.0).
    pub fn record_accuracy(&self, score: f64) {
        if let Ok(mut scores) = self.accuracy_scores.lock() {
            scores.push(score);
        }
    }

    /// Get current metrics snapshot.
    pub fn get_snapshot(&self) -> MetricsSnapshot {
        let total_saved = self.total_tokens_saved.load(Ordering::Relaxed);
        let count = self.compressions_count.load(Ordering::Relaxed);
        let errors = self.errors_count.load(Ordering::Relaxed);

        let avg_accuracy = if let Ok(scores) = self.accuracy_scores.lock() {
            if scores.is_empty() {
                0.0
            } else {
                scores.iter().sum::<f64>() / scores.len() as f64
            }
        } else {
            0.0
        };

        MetricsSnapshot {
            total_tokens_saved,
            compressions_count: count,
            errors_count: errors,
            average_accuracy: avg_accuracy,
        }
    }

    /// Reset all metrics.
    pub fn reset(&self) {
        self.total_tokens_saved.store(0, Ordering::Relaxed);
        self.compressions_count.store(0, Ordering::Relaxed);
        self.errors_count.store(0, Ordering::Relaxed);
        if let Ok(mut scores) = self.accuracy_scores.lock() {
            scores.clear();
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub total_tokens_saved: u64,
    pub compressions_count: u64,
    pub errors_count: u64,
    pub average_accuracy: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_creation() {
        let metrics = MetricsCollector::new();
        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.total_tokens_saved, 0);
        assert_eq!(snapshot.compressions_count, 0);
    }

    #[test]
    fn test_record_compression() {
        let metrics = MetricsCollector::new();
        metrics.record_compression(100);
        metrics.record_compression(50);

        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.total_tokens_saved, 150);
        assert_eq!(snapshot.compressions_count, 2);
    }

    #[test]
    fn test_record_error() {
        let metrics = MetricsCollector::new();
        metrics.record_error();
        metrics.record_error();

        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.errors_count, 2);
    }

    #[test]
    fn test_record_accuracy() {
        let metrics = MetricsCollector::new();
        metrics.record_accuracy(0.9);
        metrics.record_accuracy(0.95);
        metrics.record_accuracy(0.85);

        let snapshot = metrics.get_snapshot();
        assert!((snapshot.average_accuracy - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let metrics = MetricsCollector::new();
        metrics.record_compression(100);
        metrics.record_error();

        metrics.reset();
        let snapshot = metrics.get_snapshot();
        assert_eq!(snapshot.total_tokens_saved, 0);
        assert_eq!(snapshot.compressions_count, 0);
        assert_eq!(snapshot.errors_count, 0);
    }
}
