//! Persistence handle owned by the monolithic App.
use std::sync::Arc;
use super::store::Store;

/// Framework persistence used to map Discord custom IDs to execution records.
///
/// The App-level contract is to own the host application's database and
/// repositories as well. This skeleton currently models only its own Store
/// handle here; repository access in the Page API examples remains an API gap.
pub struct Database {
    /// Durable storage for custom-ID to execution-reference records.
    pub store: Arc<dyn Store>,
}
impl Database {
    /// Creates the framework persistence handle around the supplied Store.
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}
