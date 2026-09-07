use serde::{Deserialize, Serialize};

use crate::{Storage, StoreError, StoreResult};

/// A persisted user document — the single schema (ADR-0002). Documents are
/// JSON; functions, constants, and scripts are stored as source text so the
/// grammar can evolve without breaking saved data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Doc {
    Function(FunctionDoc),
    Constant(ConstantDoc),
    Script(ScriptDoc),
    Setting(SettingDoc),
}

/// A saved user-defined function, stored as its `def` source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDoc {
    pub name: String,
    pub source: String,
}

/// A saved user-defined constant, stored as its `const` source (ADR-0012).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstantDoc {
    pub name: String,
    pub source: String,
}

/// A saved multi-statement script, stored as source text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptDoc {
    pub name: String,
    pub source: String,
}

/// A user preference (e.g. `language`), stored as a JSON value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingDoc {
    pub key: String,
    pub value: serde_json::Value,
}

fn key(kind: &str, name: &str) -> String {
    format!("{kind}/{name}")
}

/// Typed access to the document schema over any [`Storage`].
pub struct DocStore<S: Storage> {
    storage: S,
}

impl<S: Storage> DocStore<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Access the underlying storage (e.g. for direct removal).
    pub fn storage(&self) -> &S {
        &self.storage
    }

    pub fn get_function(&self, name: &str) -> StoreResult<Option<FunctionDoc>> {
        self.get_json(&key("function", name))
    }

    pub fn put_function(&self, doc: &FunctionDoc) -> StoreResult<()> {
        self.put_json(&key("function", &doc.name), doc)
    }

    pub fn list_functions(&self) -> StoreResult<Vec<FunctionDoc>> {
        self.list_docs("function/")
    }

    pub fn get_constant(&self, name: &str) -> StoreResult<Option<ConstantDoc>> {
        self.get_json(&key("constant", name))
    }

    pub fn put_constant(&self, doc: &ConstantDoc) -> StoreResult<()> {
        self.put_json(&key("constant", &doc.name), doc)
    }

    pub fn list_constants(&self) -> StoreResult<Vec<ConstantDoc>> {
        self.list_docs("constant/")
    }

    pub fn get_script(&self, name: &str) -> StoreResult<Option<ScriptDoc>> {
        self.get_json(&key("script", name))
    }

    pub fn put_script(&self, doc: &ScriptDoc) -> StoreResult<()> {
        self.put_json(&key("script", &doc.name), doc)
    }

    pub fn list_scripts(&self) -> StoreResult<Vec<ScriptDoc>> {
        self.list_docs("script/")
    }

    pub fn get_setting(&self, name: &str) -> StoreResult<Option<serde_json::Value>> {
        match self.get_json(&key("setting", name))? {
            Some(SettingDoc { value, .. }) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    pub fn set_setting(&self, name: &str, value: serde_json::Value) -> StoreResult<()> {
        self.put_json(
            &key("setting", name),
            &SettingDoc {
                key: name.to_string(),
                value,
            },
        )
    }

    fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> StoreResult<Option<T>> {
        match self.storage.get(key)? {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map(Some)
                .map_err(|e| StoreError::Serialize(e.to_string())),
            None => Ok(None),
        }
    }

    fn put_json<T: Serialize>(&self, key: &str, value: &T) -> StoreResult<()> {
        let bytes = serde_json::to_vec(value).map_err(|e| StoreError::Serialize(e.to_string()))?;
        self.storage.put(key, &bytes)
    }

    /// List docs of one kind; unparseable documents are skipped (tolerant of
    /// schema drift).
    fn list_docs<T: serde::de::DeserializeOwned>(&self, prefix: &str) -> StoreResult<Vec<T>> {
        let mut out = Vec::new();
        for k in self.storage.list(prefix)? {
            if let Some(doc) = self.get_json(&k)? {
                out.push(doc);
            }
        }
        Ok(out)
    }
}
