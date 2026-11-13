//! Persistence used by the framework's execution router.
use std::sync::Arc;
use super::store::Store;

/// The App owns this database handle alongside its other application state.
/// Host-specific repositories belong on the concrete App, not in a framework
/// generic threaded through every Page and Component.
pub struct Database {
    pub store: Arc<dyn Store>,
}
impl Database {
    pub fn new(store: Arc<dyn Store>) -> Self {
        Self { store }
    }
}
