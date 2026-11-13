use std::sync::Arc;
use crate::app::{App, Database};
use crate::app::store::{MemoryStore, Store, StoreDurability};
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
    let store = MemoryStore::default();
    assert_eq!(store.durability(), StoreDurability::Durable);
}
