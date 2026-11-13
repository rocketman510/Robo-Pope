use std::sync::Arc;
use crate::app::{App, Database};
use crate::app::store::{FileStore, MemoryStore, Store, StoreDurability};
use super::support::app;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostDatabase { connection_name: String }

#[test]
fn app_database_is_host_owned_and_type_retrieval_is_checked() {
    // FIRST: callbacks retrieve the host's concrete repository aggregate, not Store.
    let host = HostDatabase { connection_name: "primary".into() };
    let database = Database::new(host.clone());
    assert_eq!(database.get::<HostDatabase>().unwrap(), &host);
    assert!(database.get::<String>().is_err());
}

#[test]
fn app_keeps_host_database_and_framework_store_as_separate_dependencies() {
    // FIRST: Store is framework persistence, while Database is application-owned.
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    let app = App::new(
        Arc::new(serenity::http::HttpBuilder::without_token().build()),
        Database::new(HostDatabase { connection_name: "app-db".into() }),
        store.clone(),
    );
    assert_eq!(app.db.get::<HostDatabase>().unwrap().connection_name, "app-db");
    assert_eq!(app.store().durability(), StoreDurability::ProcessLocal);
    assert!(Arc::ptr_eq(&app.store, &store));
}

#[test]
fn cloned_app_preserves_the_same_host_database_value() {
    // FIRST: cloned contexts must retain the same shared host database handle.
    let app = app();
    let cloned = app.clone();
    assert!(Arc::ptr_eq(&app.db, &cloned.db));
    assert!(cloned.db.get::<()>().is_ok());
}

#[test]
fn production_store_must_advertise_restart_durability() {
    // FIRST: production records must survive process recreation; MemoryStore is not enough.
    let path = std::env::temp_dir().join(format!(
        "robo-pope-store-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    ));
    let store = FileStore::open(&path).unwrap();
    assert_eq!(store.durability(), StoreDurability::Durable);
    drop(store);
    assert_eq!(MemoryStore::default().durability(), StoreDurability::ProcessLocal);
    let _ = std::fs::remove_file(path);
}


#[tokio::test]
async fn file_store_recovers_execution_and_pagination_records_after_reopen() {
    // FIRST: a new process can reopen the production Store and resolve the same
    // custom-ID record, including the most recently committed pagination state.
    let path = std::env::temp_dir().join(format!(
        "robo-pope-store-restart-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
    ));
    let state0 = serde_json::json!({
        "frame_id": "users", "page_index": 0, "page_count": 2
    });
    let state1 = serde_json::json!({
        "frame_id": "users", "page_index": 1, "page_count": 2
    });
    let record = crate::app::store::StoreRecord {
        custom_id: "persistent-custom-id".into(),
        definition_kind: crate::app::store::DefinitionKind::Page,
        definition_id: "persistent-page".into(),
        execution_id: "__frame:users:next".into(),
        serialized_data: br#"{"owner":42}"#.to_vec(),
        frame_identity: Some("users".into()),
        pagination_state: Some(state0.clone()),
        schema_version: 1,
    };

    {
        let store = FileStore::open(&path).unwrap();
        store.insert(record).await.unwrap();
        assert!(store.compare_and_set_pagination_state(
            &crate::app::store::CustomId("persistent-custom-id".into()),
            &state0,
            state1.clone(),
        ).await.unwrap());
    }

    {
        let reopened = FileStore::open(&path).unwrap();
        let restored = reopened.resolve(
            &crate::app::store::CustomId("persistent-custom-id".into())
        ).await.unwrap().expect("persisted custom ID must resolve after reopening");
        assert_eq!(restored.definition_id, "persistent-page");
        assert_eq!(restored.execution_id, "__frame:users:next");
        assert_eq!(restored.serialized_data, br#"{"owner":42}"#);
        assert_eq!(restored.pagination_state, Some(state1));
        assert_eq!(reopened.durability(), StoreDurability::Durable);
    }

    let _ = std::fs::remove_file(path);
}
