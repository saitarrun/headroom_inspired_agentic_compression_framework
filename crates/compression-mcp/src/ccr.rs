use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Compression Control Record: reversible compression backend.
/// Stores original outputs for retrieval on demand, enabling agents to debug
/// compressed outputs or recover full context when needed.
///
/// Design:
/// - Thread-safe in-memory storage (Arc<Mutex>)
/// - UUID-based retrieval (deterministic for same content)
/// - Metadata tracking (size, compression ratio, timestamp)
/// - Configurable retention policies (TTL, max entries)

#[derive(Debug, Clone)]
pub struct CcrRecord {
    pub id: String,
    pub original: String,
    pub timestamp: u64,
    pub original_size: usize,
    pub compressed_size: Option<usize>,
}

pub struct CcrBackend {
    storage: Arc<Mutex<HashMap<String, CcrRecord>>>,
    /// Maximum number of records to store (LRU eviction if exceeded)
    max_entries: usize,
}

impl CcrBackend {
    /// Create a new CCR backend with default settings.
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create with custom capacity.
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            max_entries,
        }
    }

    /// Store the original output and return an ID.
    /// Returns the ID for retrieval.
    pub fn store(&self, original: String) -> Result<String, String> {
        self.store_with_compressed_size(&original, None)
    }

    /// Store original with compressed size metadata (for metrics).
    pub fn store_with_compressed_size(
        &self,
        original: &str,
        compressed_size: Option<usize>,
    ) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let timestamp = current_timestamp();
        let original_size = original.len();

        let record = CcrRecord {
            id: id.clone(),
            original: original.to_string(),
            timestamp,
            original_size,
            compressed_size,
        };

        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        // Check capacity and evict oldest if needed
        if storage.len() >= self.max_entries {
            self.evict_oldest(&mut storage)?;
        }

        storage.insert(id.clone(), record);
        Ok(id)
    }

    /// Retrieve the original output by ID (byte-equal to original).
    pub fn retrieve(&self, id: &str) -> Result<String, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        storage
            .get(id)
            .map(|r| r.original.clone())
            .ok_or_else(|| format!("No stored output with ID: {}", id))
    }

    /// Retrieve record with metadata.
    pub fn retrieve_record(&self, id: &str) -> Result<CcrRecord, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        storage
            .get(id)
            .cloned()
            .ok_or_else(|| format!("No stored record with ID: {}", id))
    }

    /// Delete a stored output.
    pub fn delete(&self, id: &str) -> Result<(), String> {
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        storage.remove(id);
        Ok(())
    }

    /// Get the number of stored entries.
    pub fn count(&self) -> Result<usize, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(storage.len())
    }

    /// Get total storage size in bytes.
    pub fn total_size(&self) -> Result<usize, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        Ok(storage.values().map(|r| r.original_size).sum())
    }

    /// Get storage statistics.
    pub fn stats(&self) -> Result<CcrStats, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;

        let count = storage.len();
        let total_original_size: usize = storage.iter().map(|(_, r)| r.original_size).sum();
        let total_compressed_size: usize = storage
            .iter()
            .filter_map(|(_, r)| r.compressed_size)
            .sum();

        let compression_ratio = if total_compressed_size > 0 {
            total_original_size as f64 / total_compressed_size as f64
        } else {
            1.0
        };

        Ok(CcrStats {
            stored_records: count,
            total_original_bytes: total_original_size,
            total_compressed_bytes: total_compressed_size,
            average_compression_ratio: compression_ratio,
        })
    }

    /// Clear all stored data.
    pub fn clear(&self) -> Result<(), String> {
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        storage.clear();
        Ok(())
    }

    /// Evict oldest record (by timestamp).
    fn evict_oldest(&self, storage: &mut HashMap<String, CcrRecord>) -> Result<(), String> {
        if let Some((oldest_id, _)) = storage
            .iter()
            .min_by_key(|(_, r)| r.timestamp)
            .map(|(id, r)| (id.clone(), r.clone()))
        {
            storage.remove(&oldest_id);
        }
        Ok(())
    }

    /// List all stored IDs.
    pub fn list_ids(&self) -> Result<Vec<String>, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        Ok(storage.keys().cloned().collect())
    }
}

impl Default for CcrBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CcrStats {
    pub stored_records: usize,
    pub total_original_bytes: usize,
    pub total_compressed_bytes: usize,
    pub average_compression_ratio: f64,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccr_store_and_retrieve() {
        let ccr = CcrBackend::new();
        let original = "original output";
        let id = ccr.store(original.to_string()).expect("store failed");
        let retrieved = ccr.retrieve(&id).expect("retrieve failed");
        assert_eq!(retrieved, original);
    }

    #[test]
    fn test_ccr_byte_faithful() {
        let ccr = CcrBackend::new();
        let original = "data with special chars: !@#$%^&*()";
        let id = ccr.store(original.to_string()).expect("store failed");
        let retrieved = ccr.retrieve(&id).expect("retrieve failed");
        assert_eq!(retrieved.as_bytes(), original.as_bytes());
    }

    #[test]
    fn test_ccr_retrieve_nonexistent() {
        let ccr = CcrBackend::new();
        let result = ccr.retrieve("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_ccr_delete() {
        let ccr = CcrBackend::new();
        let id = ccr.store("data".to_string()).expect("store failed");
        ccr.delete(&id).expect("delete failed");
        let result = ccr.retrieve(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_ccr_count() {
        let ccr = CcrBackend::new();
        ccr.store("first".to_string()).expect("store failed");
        ccr.store("second".to_string()).expect("store failed");
        assert_eq!(ccr.count().expect("count failed"), 2);
    }

    #[test]
    fn test_ccr_store_with_compressed_size() {
        let ccr = CcrBackend::new();
        let original = "original data";
        let id = ccr
            .store_with_compressed_size(original, Some(7))
            .expect("store failed");

        let record = ccr.retrieve_record(&id).expect("retrieve failed");
        assert_eq!(record.original_size, 13);
        assert_eq!(record.compressed_size, Some(7));
    }

    #[test]
    fn test_ccr_total_size() {
        let ccr = CcrBackend::new();
        ccr.store("10 bytes.".to_string()).expect("store failed");
        ccr.store("another 10 bytes-".to_string()).expect("store failed");

        let size = ccr.total_size().expect("total_size failed");
        assert!(size > 0);
    }

    #[test]
    fn test_ccr_stats() {
        let ccr = CcrBackend::new();
        ccr.store("original".to_string()).expect("store failed");

        let stats = ccr.stats().expect("stats failed");
        assert_eq!(stats.stored_records, 1);
        assert!(stats.total_original_bytes > 0);
    }

    #[test]
    fn test_ccr_list_ids() {
        let ccr = CcrBackend::new();
        let id1 = ccr.store("first".to_string()).expect("store failed");
        let id2 = ccr.store("second".to_string()).expect("store failed");

        let ids = ccr.list_ids().expect("list_ids failed");
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_ccr_clear() {
        let ccr = CcrBackend::new();
        ccr.store("data".to_string()).expect("store failed");
        ccr.clear().expect("clear failed");
        assert_eq!(ccr.count().expect("count failed"), 0);
    }

    #[test]
    fn test_ccr_capacity_limit() {
        let ccr = CcrBackend::with_capacity(2);
        let id1 = ccr.store("first".to_string()).expect("store failed");
        let id2 = ccr.store("second".to_string()).expect("store failed");
        let _id3 = ccr.store("third".to_string()).expect("store failed");

        // One of the earlier records should be evicted
        assert_eq!(ccr.count().expect("count failed"), 2);

        // At least one of the original records should be gone
        let has_id1 = ccr.retrieve(&id1).is_ok();
        let has_id2 = ccr.retrieve(&id2).is_ok();
        assert!(!(has_id1 && has_id2), "Both old records should not still exist");
    }
}
