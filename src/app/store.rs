//! Persistence for deterministic custom-ID to execution-reference mappings.
use serde::{Deserialize, Serialize};
use crate::app::BoxFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefinitionKind { Page, Modal }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreRecord {
    pub custom_id: String,
    pub definition_kind: DefinitionKind,
    pub definition_id: String,
    pub execution_id: String,
    pub serialized_data: Vec<u8>,
    pub frame_identity: Option<String>,
    pub pagination_state: Option<serde_json::Value>,
    pub schema_version: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreDurability { ProcessLocal, Durable }

pub trait Store: Send + Sync {
    fn durability(&self) -> StoreDurability;
    fn insert<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>>;
    fn compare_and_set_pagination_state<'a>(
        &'a self, custom_id: &'a CustomId, expected: &'a serde_json::Value, next: serde_json::Value,
    ) -> BoxFuture<'a, anyhow::Result<bool>>;
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>>;
    fn remove<'a>(&'a self, custom_id: &'a CustomId) -> BoxFuture<'a, anyhow::Result<()>>;
}

pub fn derive_custom_id(
    definition_id: &str, execution_id: &str, serialized_data: &[u8], frame_identity: Option<&str>,
) -> anyhow::Result<CustomId> {
    let mut first = 0xcbf29ce484222325u64;
    let mut second = 0x84222325cbf29ce4u64;
    fn feed(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes { *hash ^= u64::from(*byte); *hash = hash.wrapping_mul(0x100000001b3); }
    }
    fn field(first: &mut u64, second: &mut u64, bytes: &[u8]) {
        let length = (bytes.len() as u64).to_le_bytes();
        feed(first, &length); feed(second, &length); feed(first, bytes); feed(second, bytes);
    }
    field(&mut first, &mut second, definition_id.as_bytes());
    field(&mut first, &mut second, execution_id.as_bytes());
    field(&mut first, &mut second, serialized_data);
    match frame_identity {
        Some(identity) => { field(&mut first, &mut second, &[1]); field(&mut first, &mut second, identity.as_bytes()); }
        None => field(&mut first, &mut second, &[0]),
    }
    let custom_id = format!("rp:{first:016x}{second:016x}");
    anyhow::ensure!(custom_id.chars().count() <= crate::page::limits::CUSTOM_ID_MAX_CHARS,
        "Derived Discord custom ID exceeds the maximum length");
    Ok(CustomId(custom_id))
}

#[derive(Default)]
pub struct MemoryStore { records: std::sync::RwLock<std::collections::HashMap<CustomId, StoreRecord>> }
impl Store for MemoryStore {
    fn durability(&self) -> StoreDurability { StoreDurability::ProcessLocal }

    fn insert<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let key = CustomId(record.custom_id.clone());
            let mut records = self.records.write().map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            match records.get(&key) {
                Some(existing) if existing == &record => Ok(()),
                Some(_) => anyhow::bail!("Conflicting Store record for custom ID '{}'", key.0),
                None => { records.insert(key, record); Ok(()) }
            }
        })
    }

    fn compare_and_set_pagination_state<'a>(
        &'a self, custom_id: &'a CustomId, expected: &'a serde_json::Value, next: serde_json::Value,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        Box::pin(async move {
            let mut records = self.records.write().map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            let Some(record) = records.get_mut(custom_id) else { return Ok(false); };
            if record.pagination_state.as_ref() != Some(expected) { return Ok(false); }
            record.pagination_state = Some(next);
            Ok(true)
        })
    }

    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>> {
        Box::pin(async move {
            let records = self.records.read().map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            Ok(records.get(custom_id).cloned())
        })
    }

    fn remove<'a>(&'a self, custom_id: &'a CustomId) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let mut records = self.records.write().map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            records.remove(custom_id);
            Ok(())
        })
    }
}


/// File-backed Store for production deployments that need restart durability.
///
/// Updates are written to a sibling temporary file, flushed, and atomically
/// renamed over the prior database file before becoming visible in memory.
#[derive(Clone)]
pub struct FileStore {
    path: std::sync::Arc<std::path::PathBuf>,
    records: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<CustomId, StoreRecord>>>,
}

impl FileStore {
    /// Opens an existing Store file or creates an empty Store at the supplied path.
    pub fn open(path: impl Into<std::path::PathBuf>) -> anyhow::Result<Self> {
        use std::io::Read as _;
        let path = path.into();
        let mut records = std::collections::HashMap::new();
        match std::fs::File::open(&path) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                if !bytes.is_empty() {
                    let stored: Vec<StoreRecord> = serde_json::from_slice(&bytes)
                        .map_err(|error| anyhow::anyhow!("Reading Store file '{}': {error}", path.display()))?;
                    for record in stored {
                        records.insert(CustomId(record.custom_id.clone()), record);
                    }
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        Ok(Self {
            path: std::sync::Arc::new(path),
            records: std::sync::Arc::new(std::sync::RwLock::new(records)),
        })
    }

    fn persist(
        &self,
        records: &std::collections::HashMap<CustomId, StoreRecord>,
    ) -> anyhow::Result<()> {
        use std::io::Write as _;
        let snapshot: Vec<StoreRecord> = records.values().cloned().collect();
        let bytes = serde_json::to_vec(&snapshot)?;
        let parent = self.path.parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(parent)?;
        let temporary = self.path.with_extension(format!("tmp-{}", std::process::id()));
        {
            let mut file = std::fs::OpenOptions::new()
                .create(true).truncate(true).write(true).open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        std::fs::rename(&temporary, self.path.as_path())?;
        if let Ok(directory) = std::fs::File::open(parent) {
            directory.sync_all()?;
        }
        Ok(())
    }
}

impl Store for FileStore {
    fn durability(&self) -> StoreDurability { StoreDurability::Durable }

    fn insert<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let key = CustomId(record.custom_id.clone());
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            match records.get(&key) {
                Some(existing) if existing == &record => return Ok(()),
                Some(_) => anyhow::bail!("Conflicting Store record for custom ID '{}'", key.0),
                None => {}
            }
            records.insert(key.clone(), record);
            if let Err(error) = self.persist(&records) {
                records.remove(&key);
                return Err(error);
            }
            Ok(())
        })
    }

    fn compare_and_set_pagination_state<'a>(
        &'a self,
        custom_id: &'a CustomId,
        expected: &'a serde_json::Value,
        next: serde_json::Value,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        Box::pin(async move {
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            let Some(record) = records.get_mut(custom_id) else { return Ok(false); };
            if record.pagination_state.as_ref() != Some(expected) { return Ok(false); }
            let previous = record.pagination_state.replace(next);
            if let Err(error) = self.persist(&records) {
                if let Some(record) = records.get_mut(custom_id) {
                    record.pagination_state = previous;
                }
                return Err(error);
            }
            Ok(true)
        })
    }

    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>> {
        Box::pin(async move {
            let records = self.records.read()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            Ok(records.get(custom_id).cloned())
        })
    }

    fn remove<'a>(&'a self, custom_id: &'a CustomId) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            let previous = records.remove(custom_id);
            if let Err(error) = self.persist(&records) {
                if let Some(record) = previous {
                    records.insert(custom_id.clone(), record);
                }
                return Err(error);
            }
            Ok(())
        })
    }
}
