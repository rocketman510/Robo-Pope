use crate::app::store::{
    derive_custom_id, CustomId, DefinitionKind, MemoryStore, Store, StoreRecord,
};

fn record(custom_id: &str) -> StoreRecord {
    StoreRecord {
        custom_id: custom_id.into(),
        definition_kind: DefinitionKind::Page,
        definition_id: "user-level".into(),
        execution_id: "open-profile".into(),
        serialized_data: br#"{"user_id":"42"}"#.to_vec(),
        frame_identity: None,
        pagination_state: None,
        schema_version: 1,
    }
}

#[test]
fn custom_id_derivation_is_deterministic_for_the_same_execution_identity() {
    // First-principles check: repeated render of the same definition, execution,
    // data, and Frame key must produce the same Discord custom ID.
    let first = derive_custom_id("page", "open", br#"{"id":1}"#, None).unwrap();
    let second = derive_custom_id("page", "open", br#"{"id":1}"#, None).unwrap();
    assert_eq!(first, second);
    assert!(first.0.len() <= crate::page::limits::CUSTOM_ID_MAX_CHARS);
}

#[test]
fn custom_id_changes_when_any_identity_input_changes() {
    // First-principles check: changing definition, execution, data, or Frame key
    // must not route the interaction to another instance's callback data.
    let base = derive_custom_id("page", "open", b"data", None).unwrap();
    assert_ne!(base, derive_custom_id("other-page", "open", b"data", None).unwrap());
    assert_ne!(base, derive_custom_id("page", "other-execution", b"data", None).unwrap());
    assert_ne!(base, derive_custom_id("page", "open", b"other-data", None).unwrap());
    assert_ne!(base, derive_custom_id("page", "open", b"data", Some("element-2")).unwrap());
}

#[tokio::test]
async fn memory_store_round_trips_a_record_by_custom_id() {
    // First-principles check: after a restart-safe identifier is persisted, routing
    // must be able to recover the data needed to invoke its registered execution.
    let store = MemoryStore::default();
    let record = record("abc");
    store.insert(record.clone()).await.unwrap();

    let resolved = store.resolve(&CustomId("abc".into())).await.unwrap().unwrap();
    assert_eq!(resolved.definition_id, "user-level");
    assert_eq!(resolved.execution_id, "open-profile");
    assert_eq!(resolved.serialized_data.as_slice(), br#"{"user_id":"42"}"#);
}

#[tokio::test]
async fn inserting_the_same_record_twice_is_idempotent() {
    // First-principles check: rendering identical Page data twice should reuse
    // the same execution record rather than failing or creating duplicate mappings.
    let store = MemoryStore::default();
    store.insert(record("abc")).await.unwrap();
    store.insert(record("abc")).await.unwrap();
    assert!(store.resolve(&CustomId("abc".into())).await.unwrap().is_some());
}

#[tokio::test]
async fn store_rejects_conflicting_records_for_the_same_custom_id() {
    // First-principles check: one custom ID must never ambiguously point at two
    // different pieces of callback data.
    let store = MemoryStore::default();
    store.insert(record("abc")).await.unwrap();
    let mut conflict = record("abc");
    conflict.serialized_data = b"different".to_vec();
    assert!(store.insert(conflict).await.is_err());
}

#[tokio::test]
async fn removing_a_missing_record_is_harmless() {
    // First-principles check: cleanup should be safely repeatable when records
    // have already expired or were removed by another worker.
    let store = MemoryStore::default();
    store.remove(&CustomId("missing".into())).await.unwrap();
}
