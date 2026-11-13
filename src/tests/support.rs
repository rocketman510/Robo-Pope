//! Shared test fixtures. These construct the App without calling its unfinished constructor.
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::app::{App, Database};
use crate::app::registry::{ModalRegistry, PageRegistry};
use crate::app::router::InteractionRouter;
use crate::app::store::{MemoryStore, Store};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestData {
    pub label: String,
    pub count: u32,
}
impl Default for TestData {
    fn default() -> Self {
        Self { label: "sample".into(), count: 7 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OtherData {
    pub enabled: bool,
}

/// Builds a minimal App fixture while bypassing App::new, which is under test.
pub fn app() -> App {
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    App {
        db: Arc::new(Database::new(store)),
        pages: PageRegistry::default(),
        modals: ModalRegistry::default(),
        http: Arc::new(serenity::http::HttpBuilder::without_token().build()),
        router: InteractionRouter::default(),
    }
}
