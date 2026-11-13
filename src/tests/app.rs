use std::sync::Arc;
use serenity::all::Http;
use crate::app::{App, Database};
use crate::app::registry::{ModalRegistry, PageRegistry};
use crate::app::router::InteractionRouter;
use crate::app::store::{MemoryStore, Store};
use crate::page::{PageDefinition, PageItem};
use super::support::{app, OtherData, TestData};

#[test]
fn app_clone_shares_the_same_owned_database() {
    // First-principles check: contexts and handlers may clone App, but they must
    // still observe the same application-owned database state.
    let app = app();
    let cloned = app.clone();
    assert!(Arc::ptr_eq(&app.db, &cloned.db));
}

#[test]
fn render_context_points_back_to_the_same_app_state() {
    // First-principles check: RenderCtx exists to expose App during dynamic
    // rendering; it must link to the original shared application state.
    let app = app();
    let ctx = app.render_context("user-directory");
    assert_eq!(ctx.definition_id.as_ref(), "user-directory");
    assert!(Arc::ptr_eq(&ctx.app.db, &app.db));
}

#[test]
fn app_constructor_initializes_empty_registries_and_keeps_database() {
    // First-principles check: this exercises the public constructor rather than
    // bypassing its responsibility with a hand-built App.
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    let db = Database::new(store);
    let app = App::new(Arc::new(Http::new("Bot red-phase-test")), db);
    assert!(!app.pages.contains("missing-page"));
    assert!(!app.modals.contains("missing-modal"));
}

#[test]
fn page_registry_holds_multiple_page_data_types() {
    // First-principles check: one App must register Pages with different T types,
    // while each typed query recovers its own definition.
    let registry = PageRegistry::default();
    registry.insert(Arc::new(PageDefinition::<TestData> {
        id: "users".into(), items: Vec::<PageItem<TestData>>::new(), paginate: false,
    })).unwrap();
    registry.insert(Arc::new(PageDefinition::<OtherData> {
        id: "settings".into(), items: Vec::<PageItem<OtherData>>::new(), paginate: false,
    })).unwrap();

    assert_eq!(registry.query::<TestData>("users").unwrap().id(), "users");
    assert_eq!(registry.query::<OtherData>("settings").unwrap().id(), "settings");
}

#[test]
fn page_registry_rejects_duplicate_definition_ids() {
    // First-principles check: IDs identify immutable definitions. Replacing an
    // existing ID could route old messages to the wrong callback.
    let registry = PageRegistry::default();
    let definition = Arc::new(PageDefinition::<TestData> {
        id: "unique".into(), items: Vec::new(), paginate: false,
    });
    registry.insert(definition.clone()).unwrap();
    assert!(registry.insert(definition).is_err());
}

#[test]
fn page_registry_rejects_a_query_using_the_wrong_data_type() {
    // First-principles check: registry type erasure is internal. A mismatched
    // typed query must be an error rather than an invalid downcast or panic.
    let registry = PageRegistry::default();
    registry.insert(Arc::new(PageDefinition::<TestData> {
        id: "users".into(), items: Vec::new(), paginate: false,
    })).unwrap();
    assert!(registry.query::<OtherData>("users").is_err());
}

#[test]
fn registry_reports_unknown_ids_as_errors() {
    // First-principles check: an unknown definition is a normal lookup failure,
    // not a fabricated default definition.
    let registry = PageRegistry::default();
    assert!(registry.query::<TestData>("missing").is_err());
}

#[test]
fn modal_registry_reports_unknown_definition_ids_as_errors() {
    // First-principles check: a missing Modal definition must be reported as a
    // lookup error rather than silently falling back to another form.
    let registry = ModalRegistry::default();
    assert!(registry.query::<TestData>("missing").is_err());
}
