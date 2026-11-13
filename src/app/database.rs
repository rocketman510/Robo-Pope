//! Framework persistence plus host-application database/services.
use std::ops::Deref;
use std::sync::Arc;
use super::store::Store;

/// Store persists framework execution references. E contains host-specific
/// repositories/services. Deref makes ctx.app.db.users.query(...) ergonomic.
pub struct Database<E> {
    pub store: Arc<dyn Store>,
    pub extension: E,
}
impl<E> Database<E> {
    pub fn new(store: Arc<dyn Store>, extension: E) -> Self {
        Self { store, extension }
    }
    pub fn extension(&self) -> &E {
        &self.extension
    }
    pub fn extension_mut(&mut self) -> &mut E {
        &mut self.extension
    }
}
impl<E> Deref for Database<E> {
    type Target = E;
    fn deref(&self) -> &Self::Target {
        &self.extension
    }
}
