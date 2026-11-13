//! Heterogeneous registries; concrete T is retained by typed query handles.
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::page::{PageDefinition, RegisteredPage};
use crate::page::modal::{ModalDefinition, RegisteredModal};

/// Stores Page definitions with different data types behind one registry.
///
/// The registry erases types internally; callers recover the concrete type
/// through query::<T>. Registering an existing ID is an error, not replacement.
pub struct PageRegistry {
    entries: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}
impl Clone for PageRegistry {
    fn clone(&self) -> Self { Self { entries: self.entries.clone() } }
}
impl Default for PageRegistry {
    fn default() -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())) }
    }
}
impl PageRegistry {
    /// Inserts an immutable Page definition, rejecting an already-registered ID.
    pub fn insert<T: Send + Sync + 'static>(&self, definition: Arc<PageDefinition<T>>) -> anyhow::Result<()> {
        let id = definition.id.clone();
        let mut entries = self.entries.write()
            .map_err(|_| anyhow::anyhow!("Page registry lock was poisoned"))?;
        anyhow::ensure!(!entries.contains_key(&id), "Page ID is already registered: {id}");
        entries.insert(id, definition);
        Ok(())
    }
    /// Retrieves a Page by ID and verifies that its stored data type is T.
    /// Returns an error for an unknown ID or a type mismatch.
    pub fn query<T: Send + Sync + 'static>(&self, id: &str) -> anyhow::Result<RegisteredPage<T>> {
        let entries = self.entries.read()
            .map_err(|_| anyhow::anyhow!("Page registry lock was poisoned"))?;
        let definition = entries.get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No Page is registered with ID: {id}"))?;
        let definition = Arc::downcast::<PageDefinition<T>>(definition)
            .map_err(|_| anyhow::anyhow!("Page '{id}' was registered with a different data type"))?;
        Ok(RegisteredPage { definition })
    }
    /// Reports whether any Page definition is registered under this ID.
    pub fn contains(&self, id: &str) -> bool {
        self.entries.read()
            .map(|entries| entries.contains_key(id))
            .unwrap_or(false)
    }
}

/// Stores Modal definitions with different data types behind one registry.
///
/// Duplicate IDs are rejected. Typed queries fail rather than returning a
/// Modal definition whose data type differs from the requested type.
pub struct ModalRegistry {
    entries: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
}
impl Clone for ModalRegistry {
    fn clone(&self) -> Self { Self { entries: self.entries.clone() } }
}
impl Default for ModalRegistry {
    fn default() -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())) }
    }
}
impl ModalRegistry {
    /// Inserts an immutable Modal definition, rejecting an already-registered ID.
    pub fn insert<T: Send + Sync + 'static>(&self, definition: Arc<ModalDefinition<T>>) -> anyhow::Result<()> {
        let id = definition.id.clone();
        let mut entries = self.entries.write()
            .map_err(|_| anyhow::anyhow!("Modal registry lock was poisoned"))?;
        anyhow::ensure!(!entries.contains_key(&id), "Modal ID is already registered: {id}");
        entries.insert(id, definition);
        Ok(())
    }
    /// Retrieves a Modal by ID and verifies that its stored data type is T.
    /// Returns an error for an unknown ID or a type mismatch.
    pub fn query<T: Send + Sync + 'static>(&self, id: &str) -> anyhow::Result<RegisteredModal<T>> {
        let entries = self.entries.read()
            .map_err(|_| anyhow::anyhow!("Modal registry lock was poisoned"))?;
        let definition = entries.get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No Modal is registered with ID: {id}"))?;
        let definition = Arc::downcast::<ModalDefinition<T>>(definition)
            .map_err(|_| anyhow::anyhow!("Modal '{id}' was registered with a different data type"))?;
        Ok(RegisteredModal { definition })
    }
    /// Reports whether any Modal definition is registered under this ID.
    pub fn contains(&self, id: &str) -> bool {
        self.entries.read()
            .map(|entries| entries.contains_key(id))
            .unwrap_or(false)
    }
}
