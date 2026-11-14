use crate::app::store::{
    derive_custom_id, CustomId, FileStore, MemoryStore, RoutingRecord, Store,
};

fn record(hash: &str) -> RoutingRecord {
    RoutingRecord {
        hash: hash.into(),
        definition_id: "user-level".into(),
        execution_id: "open-profile".into(),
        serialized_data: br#"{"user_id":"42"}"#.to_vec(),
    }
}

#[test]
fn custom_id_derivation_is_deterministic_for_the_same_execution_identity() {
    // FIRST: the same definition ID, execution ID, and serialized data produce
    // the same Discord custom ID on every render and after process restarts.
    let first = derive_custom_id("page", "open", br#"{"id":1}"#).unwrap();
    let second = derive_custom_id("page", "open", br#"{"id":1}"#).unwrap();
    assert_eq!(first, second);
    assert!(first.0.len() <= crate::page::limits::CUSTOM_ID_MAX_CHARS);
}

#[test]
fn custom_id_changes_when_any_routing_identity_input_changes() {
    // FIRST: the hash is derived from exactly the definition ID, execution ID,
    // and serialized data. No Frame path, user ID, message ID, or timestamp is
    // an extra input to custom-ID derivation.
    let base = derive_custom_id("page", "open", b"data").unwrap();
    assert_ne!(base, derive_custom_id("other-page", "open", b"data").unwrap());
    assert_ne!(base, derive_custom_id("page", "other-execution", b"data").unwrap());
    assert_ne!(base, derive_custom_id("page", "open", b"other-data").unwrap());
}

#[tokio::test]
async fn memory_store_round_trips_a_four_field_routing_record_by_hash() {
    // FIRST: a resolved hash returns exactly the definition, execution, and
    // serialized data needed by the registered definition to dispatch.
    let store = MemoryStore::default();
    store.insert(record("rp:abc")).await.unwrap();

    let resolved = store.resolve(&CustomId("rp:abc".into())).await.unwrap().unwrap();
    assert_eq!(resolved.hash, "rp:abc");
    assert_eq!(resolved.definition_id, "user-level");
    assert_eq!(resolved.execution_id, "open-profile");
    assert_eq!(resolved.serialized_data.as_slice(), br#"{"user_id":"42"}"#);
}

#[tokio::test]
async fn inserting_the_same_routing_record_twice_is_idempotent() {
    // FIRST: rendering identical definition, execution, and data reuses one
    // logical routing record.
    let store = MemoryStore::default();
    store.insert(record("rp:abc")).await.unwrap();
    store.insert(record("rp:abc")).await.unwrap();
    assert!(store.resolve(&CustomId("rp:abc".into())).await.unwrap().is_some());
}

#[tokio::test]
async fn store_rejects_conflicting_records_for_the_same_hash() {
    // FIRST: if one hash is presented with different identity fields, the Store
    // reports a collision rather than overwriting the first callback mapping.
    let store = MemoryStore::default();
    store.insert(record("rp:abc")).await.unwrap();

    let mut conflict = record("rp:abc");
    conflict.serialized_data = b"different".to_vec();

    assert!(store.insert(conflict).await.is_err());
}

#[tokio::test]
async fn removing_a_missing_routing_record_is_harmless() {
    // FIRST: an explicit removal request for a missing hash is idempotent.
    // The framework does not remove records automatically through TTL expiry.
    let store = MemoryStore::default();
    store.remove(&CustomId("missing".into())).await.unwrap();
}


fn unique_store_path(label: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "robo-pope-{label}-{}-{nonce}.json",
        std::process::id(),
    ))
}

#[test]
fn file_store_rejects_conflicting_duplicate_hashes_when_loading() {
    // Page API §11: a stored hash collision must be reported, never overwritten.
    let path = unique_store_path("collision-load");
    let first = record("rp:duplicate");
    let mut conflicting = first.clone();
    conflicting.execution_id = "different-execution".into();

    std::fs::write(
        &path,
        serde_json::to_vec(&[first, conflicting]).unwrap(),
    ).unwrap();

    let result = FileStore::open(&path);
    assert!(result.is_err(), "conflicting records with one hash must fail to load");
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn file_store_idempotent_insert_preserves_record_when_persist_fails() {
    // Page API §11: reinserting an identical identity is idempotent. A failed
    // snapshot write must not erase the already-valid in-memory routing record.
    let path = unique_store_path("duplicate-write");
    let store = FileStore::open(&path).unwrap();
    let original = record("rp:existing");
    store.insert(original.clone()).await.unwrap();

    // Make the destination impossible to replace while keeping the FileStore
    // instance and its already-loaded in-memory record alive.
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();

    assert!(store.insert(original.clone()).await.is_err());
    let resolved = store.resolve(&CustomId("rp:existing".into())).await.unwrap();
    assert_eq!(
        resolved,
        Some(original),
        "a failed idempotent write must not roll back a record that predated it",
    );

    let _ = std::fs::remove_dir_all(&path);
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    let _ = std::fs::remove_file(temporary);
}
