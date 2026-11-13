//! Persistence contract for interactive execution references.
//!
//! Discord returns custom_id when a component is used. Store resolves that
//! identifier to a record containing definition kind (Page or Modal),
//! definition ID, execution ID, serialized instance data, and optional
//! Frame/element identity for interactive content generated inside a Frame.
//!
//! Custom IDs must fit Discord's 100-character limit. Hash inputs must be
//! unambiguous and versioned, using deterministic serialization and a stable
//! hash such as BLAKE3. A hash is an identifier, not an authorization token.
//!
//! Store is an abstraction, not proof of durable persistence. An in-memory
//! implementation is useful for tests and local development but does not
//! survive restarts. Production must supply a durable backend. Re-rendering
//! the same identity should be idempotent; a conflicting record must not
//! silently redirect an old custom ID to different behavior. Define expiry
//! and cleanup so deleted messages do not leave unbounded records.
//!
//! Intended shape (illustrative):
//! \`\`\`rust,ignore
//! struct StoreRecord {
//!     definition_kind: DefinitionKind,
//!     definition_id: String,
//!     execution_id: String,
//!     instance_data: Vec<u8>,
//!     frame_identity: Option<FrameExecutionIdentity>,
//! }
//! trait Store {
//!     async fn insert(&self, id: CustomId, record: StoreRecord) -> Result<()>;
//!     async fn resolve(&self, id: &CustomId) -> Result<Option<StoreRecord>>;
//!     async fn delete(&self, id: &CustomId) -> Result<()>;
//! }
//! \`\`\`
//!
//! TODO: Define Store, StoreRecord, DefinitionKind, and MemoryStore.
//! TODO: Define canonical serialization, identity versioning, and hashing.
//! TODO: Include Page/Modal kind and Frame key in identity inputs.
//! TODO: Define conflict, expiry, deletion, and migration semantics.
//! TODO: Surface backend errors; never swallow failures or poisoned locks.
