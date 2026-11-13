//! Persistence for deterministic custom-ID to execution-reference mappings.
use serde::{Deserialize, Serialize};
use crate::app::BoxFuture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefinitionKind {
    Page,
    Modal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Durable adapters must survive restarts. MemoryStore is test/development-only.
pub trait Store: Send + Sync {
    fn insert<'a>(&'a self, record: StoreRecord)
        -> BoxFuture<'a, anyhow::Result<()>>;
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>>;
    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>>;
}

/// Stable identity includes definition, execution, serialized data, and an
/// optional stable Frame identity. It is an identifier, never authorization.
pub fn derive_custom_id(
    definition_id: &str,
    execution_id: &str,
    serialized_data: &[u8],
    frame_identity: Option<&str>,
) -> anyhow::Result<CustomId> {
    todo!("canonicalize identity inputs, hash deterministically, enforce Discord length")
}

#[derive(Default)]
pub struct MemoryStore {
    records: std::sync::RwLock<std::collections::HashMap<CustomId, StoreRecord>>,
}
impl Store for MemoryStore {
    fn insert<'a>(&'a self, record: StoreRecord)
        -> BoxFuture<'a, anyhow::Result<()>>
    {
        Box::pin(async move { todo!("define idempotent insert and conflicting-record behavior") })
    }
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>>
    {
        Box::pin(async move { todo!("resolve a stored custom ID") })
    }
    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>>
    {
        Box::pin(async move { todo!("remove an expired or deleted execution record") })
    }
}
