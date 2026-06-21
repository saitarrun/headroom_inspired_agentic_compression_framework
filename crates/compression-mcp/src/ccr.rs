use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Compression Control Record: reversible compression backend.
/// Stores original outputs for retrieval on demand.
pub struct CcrBackend {
    storage: Arc<Mutex<HashMap<String, String>>>,
}

impl CcrBackend {
    /// Create a new CCR backend.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Store the original output and return an ID.
    pub fn store(&self, original: String) -> Result<String, String> {
        let id = Uuid::new_v4().to_string();
        let mut storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        storage.insert(id.clone(), original);
        Ok(id)
    }

    /// Retrieve the original output by ID.
    pub fn retrieve(&self, id: &str) -> Result<String, String> {
        let storage = self
            .storage
            .lock()
            .map_err(|e| format!("Lock error: {}", e))?;
        storage
            .get(id)
            .cloned()
            .ok_or_else(|| format!("No stored output with ID: {}", id))
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
}

impl Default for CcrBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ccr_store_and_retrieve() {
        let ccr = CcrBackend::new();
        let original = "original output".to_string();
        let id = ccr.store(original.clone()).expect("store failed");
        let retrieved = ccr.retrieve(&id).expect("retrieve failed");
        assert_eq!(retrieved, original);
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
}
