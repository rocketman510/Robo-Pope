//! Durable and process-local persistence for deterministic Discord custom IDs.
use serde::{Deserialize, Serialize};
use rusqlite::OptionalExtension as _;
use crate::app::BoxFuture;

/// Public, application-independent route mapping persisted by a Store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingRecord {
    pub hash: String,
    pub definition_id: String,
    pub execution_id: String,
    pub serialized_data: Vec<u8>,
}

/// Internal route metadata used by the renderer for framework-managed controls.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefinitionKind { Page, Modal }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreDurability { ProcessLocal, Durable }

impl From<RoutingRecord> for StoreRecord {
    fn from(record: RoutingRecord) -> Self {
        Self {
            custom_id: record.hash,
            definition_kind: DefinitionKind::Page,
            definition_id: record.definition_id,
            execution_id: record.execution_id,
            serialized_data: record.serialized_data,
            frame_identity: None,
            pagination_state: None,
            schema_version: 1,
        }
    }
}

impl From<StoreRecord> for RoutingRecord {
    fn from(record: StoreRecord) -> Self {
        Self {
            hash: record.custom_id,
            definition_id: record.definition_id,
            execution_id: record.execution_id,
            serialized_data: record.serialized_data,
        }
    }
}

/// Store interface. RoutingRecord is the compact public contract; the
/// extended methods are reserved for internal framework-managed controls.
pub trait Store: Send + Sync {
    fn durability(&self) -> StoreDurability;

    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>>;
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>>;

    fn insert_record<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>>;
    fn resolve_record<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>>;

    fn compare_and_set_pagination_state<'a>(
        &'a self,
        custom_id: &'a CustomId,
        expected: &'a serde_json::Value,
        next: serde_json::Value,
    ) -> BoxFuture<'a, anyhow::Result<bool>>;

    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>>;
}

/// Derives the Discord custom ID from exactly the three public routing inputs.
pub fn derive_custom_id(
    definition_id: &str,
    execution_id: &str,
    serialized_data: &[u8],
) -> anyhow::Result<CustomId> {
    derive_custom_id_with_frame(definition_id, execution_id, serialized_data, None)
}

/// Internal derivation variant for legacy framework-managed pagination controls.
pub fn derive_custom_id_with_frame(
    definition_id: &str,
    execution_id: &str,
    serialized_data: &[u8],
    frame_identity: Option<&str>,
) -> anyhow::Result<CustomId> {
    let mut first = 0xcbf29ce484222325u64;
    let mut second = 0x84222325cbf29ce4u64;

    fn feed(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(0x100000001b3);
        }
    }

    fn field(first: &mut u64, second: &mut u64, bytes: &[u8]) {
        let length = (bytes.len() as u64).to_le_bytes();
        feed(first, &length);
        feed(second, &length);
        feed(first, bytes);
        feed(second, bytes);
    }

    field(&mut first, &mut second, definition_id.as_bytes());
    field(&mut first, &mut second, execution_id.as_bytes());
    field(&mut first, &mut second, serialized_data);
    match frame_identity {
        Some(identity) => {
            field(&mut first, &mut second, &[1]);
            field(&mut first, &mut second, identity.as_bytes());
        }
        None => field(&mut first, &mut second, &[0]),
    }

    let custom_id = format!("rp:{first:016x}{second:016x}");
    anyhow::ensure!(
        custom_id.chars().count() <= crate::page::limits::CUSTOM_ID_MAX_CHARS,
        "Derived Discord custom ID exceeds the maximum length"
    );
    Ok(CustomId(custom_id))
}

fn routing_record(record: &StoreRecord) -> RoutingRecord {
    RoutingRecord {
        hash: record.custom_id.clone(),
        definition_id: record.definition_id.clone(),
        execution_id: record.execution_id.clone(),
        serialized_data: record.serialized_data.clone(),
    }
}

fn insert_memory(
    records: &mut std::collections::HashMap<CustomId, StoreRecord>,
    record: StoreRecord,
) -> anyhow::Result<()> {
    let key = CustomId(record.custom_id.clone());
    match records.get(&key) {
        Some(existing) if existing == &record => Ok(()),
        Some(_) => anyhow::bail!("Conflicting Store record for custom ID '{}'", key.0),
        None => {
            records.insert(key, record);
            Ok(())
        }
    }
}

/// In-memory Store for tests and process-local deployments.
#[derive(Default)]
pub struct MemoryStore {
    records: std::sync::RwLock<std::collections::HashMap<CustomId, StoreRecord>>,
}

impl Store for MemoryStore {
    fn durability(&self) -> StoreDurability { StoreDurability::ProcessLocal }

    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        self.insert_record(record.into())
    }

    fn insert_record<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            insert_memory(&mut records, record)
        })
    }

    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>> {
        Box::pin(async move {
            let records = self.records.read()
                .map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            Ok(records.get(custom_id).map(routing_record))
        })
    }

    fn resolve_record<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>> {
        Box::pin(async move {
            let records = self.records.read()
                .map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            Ok(records.get(custom_id).cloned())
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
                .map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            let Some(record) = records.get_mut(custom_id) else { return Ok(false); };
            if record.pagination_state.as_ref() != Some(expected) { return Ok(false); }
            record.pagination_state = Some(next);
            Ok(true)
        })
    }

    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            records.remove(custom_id);
            Ok(())
        })
    }
}

/// SQLite Store for restart-durable routing records and framework metadata.
pub struct SqliteStore {
    connection: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

impl SqliteStore {
    /// Opens or creates a SQLite database and initializes its routing table.
    pub async fn open(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let connection = rusqlite::Connection::open(path)?;
        connection.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = FULL;
             CREATE TABLE IF NOT EXISTS routing_records (
                 hash TEXT PRIMARY KEY NOT NULL,
                 record_json TEXT NOT NULL
             );"
        )?;
        Ok(Self {
            connection: std::sync::Arc::new(std::sync::Mutex::new(connection)),
        })
    }

    fn put(&self, record: StoreRecord) -> BoxFuture<'_, anyhow::Result<()>> {
        let connection = self.connection.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let json = serde_json::to_string(&record)?;
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                let existing: Option<String> = connection.query_row(
                    "SELECT record_json FROM routing_records WHERE hash = ?1",
                    [&record.custom_id],
                    |row| row.get(0),
                ).optional()?;
                match existing {
                    Some(existing) => {
                        let existing: StoreRecord = serde_json::from_str(&existing)?;
                        anyhow::ensure!(
                            existing == record,
                            "Conflicting Store record for custom ID '{}'",
                            record.custom_id
                        );
                        Ok(())
                    }
                    None => {
                        connection.execute(
                            "INSERT INTO routing_records (hash, record_json) VALUES (?1, ?2)",
                            rusqlite::params![record.custom_id, json],
                        )?;
                        Ok(())
                    }
                }
            }).await?
        })
    }

    fn get_record(&self, custom_id: &CustomId)
        -> BoxFuture<'_, anyhow::Result<Option<StoreRecord>>> {
        let connection = self.connection.clone();
        let hash = custom_id.0.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Option<StoreRecord>> {
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                let json: Option<String> = connection.query_row(
                    "SELECT record_json FROM routing_records WHERE hash = ?1",
                    [&hash],
                    |row| row.get(0),
                ).optional()?;
                json.map(|value| serde_json::from_str(&value).map_err(anyhow::Error::from)).transpose()
            }).await?
        })
    }
}

impl Store for SqliteStore {
    fn durability(&self) -> StoreDurability { StoreDurability::Durable }

    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        self.put(record.into())
    }

    fn insert_record<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        self.put(record)
    }

    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>> {
        let future = self.get_record(custom_id);
        Box::pin(async move { Ok(future.await?.map(RoutingRecord::from)) })
    }

    fn resolve_record<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>> {
        self.get_record(custom_id)
    }

    fn compare_and_set_pagination_state<'a>(
        &'a self,
        custom_id: &'a CustomId,
        expected: &'a serde_json::Value,
        next: serde_json::Value,
    ) -> BoxFuture<'a, anyhow::Result<bool>> {
        let connection = self.connection.clone();
        let hash = custom_id.0.clone();
        let expected = expected.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<bool> {
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                let json: Option<String> = connection.query_row(
                    "SELECT record_json FROM routing_records WHERE hash = ?1",
                    [&hash],
                    |row| row.get(0),
                ).optional()?;
                let Some(json) = json else { return Ok(false); };
                let mut record: StoreRecord = serde_json::from_str(&json)?;
                if record.pagination_state.as_ref() != Some(&expected) { return Ok(false); }
                record.pagination_state = Some(next);
                connection.execute(
                    "UPDATE routing_records SET record_json = ?2 WHERE hash = ?1",
                    rusqlite::params![hash, serde_json::to_string(&record)?],
                )?;
                Ok(true)
            }).await?
        })
    }

    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>> {
        let connection = self.connection.clone();
        let hash = custom_id.0.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                connection.execute(
                    "DELETE FROM routing_records WHERE hash = ?1",
                    [&hash],
                )?;
                Ok(())
            }).await?
        })
    }
}

/// File-backed Store retained for deployments that prefer atomic JSON snapshots.
#[derive(Clone)]
pub struct FileStore {
    path: std::sync::Arc<std::path::PathBuf>,
    records: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<CustomId, StoreRecord>>>,
}

impl FileStore {
    pub fn open(path: impl Into<std::path::PathBuf>) -> anyhow::Result<Self> {
        use std::io::Read as _;
        let path = path.into();
        let mut records = std::collections::HashMap::new();
        match std::fs::File::open(&path) {
            Ok(mut file) => {
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                if !bytes.is_empty() {
                    let stored: Vec<StoreRecord> = serde_json::from_slice(&bytes)?;
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

    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        self.insert_record(record.into())
    }

    fn insert_record<'a>(&'a self, record: StoreRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let key = CustomId(record.custom_id.clone());
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            if let Err(error) = insert_memory(&mut records, record) {
                return Err(error);
            }
            if let Err(error) = self.persist(&records) {
                records.remove(&key);
                return Err(error);
            }
            Ok(())
        })
    }

    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>> {
        let future = self.resolve_record(custom_id);
        Box::pin(async move { Ok(future.await?.map(RoutingRecord::from)) })
    }

    fn resolve_record<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<StoreRecord>>> {
        Box::pin(async move {
            let records = self.records.read()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            Ok(records.get(custom_id).cloned())
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

    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            let previous = records.remove(custom_id);
            if let Err(error) = self.persist(&records) {
                if let Some(record) = previous { records.insert(custom_id.clone(), record); }
                return Err(error);
            }
            Ok(())
        })
    }
}
