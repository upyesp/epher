use std::cell::RefCell;
use std::collections::HashMap;

use crate::{Storage, StoreResult};

/// An in-memory [`Storage`] — the test fake and default fallback.
#[derive(Debug, Default)]
pub struct MemoryStore {
    map: RefCell<HashMap<String, Vec<u8>>>,
}

impl Storage for MemoryStore {
    fn get(&self, key: &str) -> StoreResult<Option<Vec<u8>>> {
        Ok(self.map.borrow().get(key).cloned())
    }

    fn put(&self, key: &str, value: &[u8]) -> StoreResult<()> {
        self.map
            .borrow_mut()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn list(&self, prefix: &str) -> StoreResult<Vec<String>> {
        let mut keys: Vec<String> = self
            .map
            .borrow()
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        keys.sort();
        Ok(keys)
    }

    fn remove(&self, key: &str) -> StoreResult<()> {
        self.map.borrow_mut().remove(key);
        Ok(())
    }
}
