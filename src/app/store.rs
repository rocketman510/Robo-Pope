//! Persistent routing records keyed by deterministic Discord custom IDs.
use serde::{Deserialize, Serialize};
use rusqlite::OptionalExtension as _;
use crate::app::BoxFuture;

/// The complete public routing record: its hash and the three hash inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoutingRecord {
    pub hash: String,
    pub definition_id: String,
    pub execution_id: String,
    pub serialized_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CustomId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreDurability { ProcessLocal, Durable }

/// Store for persistent callback routing. All implementations store the same
/// four-field RoutingRecord without framework-specific metadata.
pub trait Store: Send + Sync {
    fn durability(&self) -> StoreDurability;
    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>>;
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>>;
    fn remove<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<()>>;
}

/// Derives a stable custom ID from exactly definition ID, execution ID, and
/// serialized Page or Modal data.
pub fn derive_custom_id(
    definition_id: &str,
    execution_id: &str,
    serialized_data: &[u8],
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
        feed(first, &length); feed(second, &length);
        feed(first, bytes); feed(second, bytes);
    }
    field(&mut first, &mut second, definition_id.as_bytes());
    field(&mut first, &mut second, execution_id.as_bytes());
    field(&mut first, &mut second, serialized_data);
    let custom_id = format!("rp:{first:016x}{second:016x}");
    anyhow::ensure!(
        custom_id.chars().count() <= crate::page::limits::CUSTOM_ID_MAX_CHARS,
        "Derived Discord custom ID exceeds the maximum length"
    );
    Ok(CustomId(custom_id))
}

fn insert_memory(
    records: &mut std::collections::HashMap<CustomId, RoutingRecord>,
    record: RoutingRecord,
) -> anyhow::Result<()> {
    let key = CustomId(record.hash.clone());
    match records.get(&key) {
        Some(existing) if existing == &record => Ok(()),
        Some(_) => anyhow::bail!("Hash collision for routing record '{}'", key.0),
        None => { records.insert(key, record); Ok(()) }
    }
}

#[derive(Default)]
pub struct MemoryStore {
    records: std::sync::RwLock<std::collections::HashMap<CustomId, RoutingRecord>>,
}
impl Store for MemoryStore {
    fn durability(&self) -> StoreDurability { StoreDurability::ProcessLocal }
    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>> {
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
            Ok(records.get(custom_id).cloned())
        })
    }
    fn remove<'a>(&'a self, custom_id: &'a CustomId) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("Memory Store lock was poisoned"))?;
            records.remove(custom_id); Ok(())
        })
    }
}

/// SQLite Store for restart-durable four-field routing records.
pub struct SqliteStore {
    connection: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}
impl SqliteStore {
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
        Ok(Self { connection: std::sync::Arc::new(std::sync::Mutex::new(connection)) })
    }
    fn put(&self, record: RoutingRecord) -> BoxFuture<'_, anyhow::Result<()>> {
        let connection = self.connection.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let json = serde_json::to_string(&record)?;
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                let existing: Option<String> = connection.query_row(
                    "SELECT record_json FROM routing_records WHERE hash = ?1",
                    [&record.hash], |row| row.get(0),
                ).optional()?;
                match existing {
                    Some(existing) => {
                        let existing: RoutingRecord = serde_json::from_str(&existing)?;
                        anyhow::ensure!(existing == record,
                            "Hash collision for routing record '{}'", record.hash);
                        Ok(())
                    }
                    None => {
                        connection.execute(
                            "INSERT INTO routing_records (hash, record_json) VALUES (?1, ?2)",
                            rusqlite::params![record.hash, json],
                        )?;
                        Ok(())
                    }
                }
            }).await?
        })
    }
    fn get_record(&self, custom_id: &CustomId)
        -> BoxFuture<'_, anyhow::Result<Option<RoutingRecord>>> {
        let connection = self.connection.clone();
        let hash = custom_id.0.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<Option<RoutingRecord>> {
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                let json: Option<String> = connection.query_row(
                    "SELECT record_json FROM routing_records WHERE hash = ?1",
                    [&hash], |row| row.get(0),
                ).optional()?;
                json.map(|value| serde_json::from_str(&value).map_err(anyhow::Error::from)).transpose()
            }).await?
        })
    }
}
impl Store for SqliteStore {
    fn durability(&self) -> StoreDurability { StoreDurability::Durable }
    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        self.put(record)
    }
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>> { self.get_record(custom_id) }
    fn remove<'a>(&'a self, custom_id: &'a CustomId) -> BoxFuture<'a, anyhow::Result<()>> {
        let connection = self.connection.clone();
        let hash = custom_id.0.clone();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
                let connection = connection.lock()
                    .map_err(|_| anyhow::anyhow!("SQLite Store lock was poisoned"))?;
                connection.execute("DELETE FROM routing_records WHERE hash = ?1", [&hash])?;
                Ok(())
            }).await?
        })
    }
}

/// File-backed Store retained for deployments that prefer atomic JSON snapshots.
#[derive(Clone)]
pub struct FileStore {
    path: std::sync::Arc<std::path::PathBuf>,
    records: std::sync::Arc<std::sync::RwLock<std::collections::HashMap<CustomId, RoutingRecord>>>,
}
impl FileStore {
    pub fn open(path: impl Into<std::path::PathBuf>) -> anyhow::Result<Self> {
        use std::io::Read as _;
        let path = path.into();
        let mut records = std::collections::HashMap::new();
        match std::fs::File::open(&path) {
            Ok(mut file) => {
                let mut bytes = Vec::new(); file.read_to_end(&mut bytes)?;
                if !bytes.is_empty() {
                    let stored: Vec<RoutingRecord> = serde_json::from_slice(&bytes)?;
                    for record in stored { records.insert(CustomId(record.hash.clone()), record); }
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
    fn persist(&self, records: &std::collections::HashMap<CustomId, RoutingRecord>) -> anyhow::Result<()> {
        use std::io::Write as _;
        let snapshot: Vec<RoutingRecord> = records.values().cloned().collect();
        let bytes = serde_json::to_vec(&snapshot)?;
        let parent = self.path.parent().filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        std::fs::create_dir_all(parent)?;
        let temporary = self.path.with_extension(format!("tmp-{}", std::process::id()));
        {
            let mut file = std::fs::OpenOptions::new()
                .create(true).truncate(true).write(true).open(&temporary)?;
            file.write_all(&bytes)?; file.sync_all()?;
        }
        std::fs::rename(&temporary, self.path.as_path())?;
        if let Ok(directory) = std::fs::File::open(parent) { directory.sync_all()?; }
        Ok(())
    }
}
impl Store for FileStore {
    fn durability(&self) -> StoreDurability { StoreDurability::Durable }
    fn insert<'a>(&'a self, record: RoutingRecord) -> BoxFuture<'a, anyhow::Result<()>> {
        Box::pin(async move {
            let key = CustomId(record.hash.clone());
            let mut records = self.records.write()
                .map_err(|_| anyhow::anyhow!("File Store lock was poisoned"))?;
            insert_memory(&mut records, record)?;
            if let Err(error) = self.persist(&records) {
                records.remove(&key); return Err(error);
            }
            Ok(())
        })
    }
    fn resolve<'a>(&'a self, custom_id: &'a CustomId)
        -> BoxFuture<'a, anyhow::Result<Option<RoutingRecord>>> {
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
                if let Some(record) = previous { records.insert(custom_id.clone(), record); }
                return Err(error);
            }
            Ok(())
        })
    }
}
