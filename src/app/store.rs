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
