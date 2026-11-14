use std::sync::Arc;

use crate::app::App;
use crate::app::store::{MemoryStore, SqliteStore, Store};
use super::support::app;

#[derive(Debug, Clone, PartialEq, Eq)]
struct HostDatabase {
    pub messages: Arc<str>,
    pub users: Arc<str>,
}

fn http() -> Arc<serenity::all::Http> {
    Arc::new(serenity::http::HttpBuilder::without_token().build())
}

#[test]
fn app_keeps_the_concrete_host_database_type() {
    // FIRST: callbacks access the application's own concrete database type
    // through App<D>, not through a type-erased get::<T>() lookup.
    let database = Arc::new(HostDatabase {
        messages: Arc::from("message repository"),
        users: Arc::from("user repository"),
    });
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    let app: App<Arc<HostDatabase>> = App::new(http(), database.clone(), store);

    assert_eq!(app.db.messages.as_ref(), "message repository");
    assert_eq!(app.db.users.as_ref(), "user repository");
    assert!(Arc::ptr_eq(&app.db, &database));
}

#[test]
fn app_keeps_host_database_and_framework_store_as_separate_dependencies() {
    // FIRST: the application database is strongly typed D; the Store is the
    // framework's separate persistence dependency.
    let database = Arc::new(HostDatabase {
        messages: Arc::from("messages"),
        users: Arc::from("users"),
    });
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    let app: App<Arc<HostDatabase>> = App::new(http(), database.clone(), store.clone());

    assert!(Arc::ptr_eq(&app.db, &database));
    assert!(Arc::ptr_eq(&app.store, &store));
    assert_eq!(app.db.messages.as_ref(), "messages");
}

#[test]
fn cloned_app_preserves_the_same_typed_database_value() {
    // FIRST: cloning App must preserve the concrete database handle, not
    // replace it with a type-erased or string-indexed database.
    let app = app();
    let cloned = app.clone();
    assert!(Arc::ptr_eq(&app.db, &cloned.db));
}

#[tokio::test]
async fn sqlite_store_recovers_the_four_field_routing_record_after_reopen() {
    // FIRST: the framework Store is SQLite-backed and its routing records
    // survive closing and reopening the Store.
    let path = std::env::temp_dir().join(format!(
        "robo-pope-routing-{}-{}.sqlite",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let record = crate::app::store::RoutingRecord {
        hash: "rp:stable-hash".into(),
        definition_id: "persistent-page".into(),
        execution_id: "open-profile".into(),
        serialized_data: br#"{"owner":42}"#.to_vec(),
    };

    {
        let store = SqliteStore::open(&path).await.unwrap();
        store.insert(record.clone()).await.unwrap();
    }

    {
        let reopened = SqliteStore::open(&path).await.unwrap();
        let restored = reopened
            .resolve(&crate::app::store::CustomId("rp:stable-hash".into()))
            .await
            .unwrap()
            .expect("the routing record must survive reopening");

        assert_eq!(restored.hash, "rp:stable-hash");
        assert_eq!(restored.definition_id, "persistent-page");
        assert_eq!(restored.execution_id, "open-profile");
        assert_eq!(restored.serialized_data, br#"{"owner":42}"#);
    }

    let _ = std::fs::remove_file(path);
}
