use crate::app::store::{
    derive_custom_id, CustomId, MemoryStore, RoutingRecord, Store,
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
