//! Heterogeneous registries; concrete T is retained by typed query handles.
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::page::{PageDefinition, RegisteredPage};
use crate::page::modal::{ModalDefinition, RegisteredModal};

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
    pub fn insert<T: Send + Sync + 'static>(&self, definition: Arc<PageDefinition<T>>) -> anyhow::Result<()> {
        todo!("insert atomically and reject duplicate Page IDs")
    }
    pub fn query<T: Send + Sync + 'static>(&self, id: &str) -> anyhow::Result<RegisteredPage<T>> {
        todo!("downcast internally and return a correctly typed handle")
    }
    pub fn contains(&self, id: &str) -> bool {
        todo!("check definition presence")
    }
}

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
    pub fn insert<T: Send + Sync + 'static>(&self, definition: Arc<ModalDefinition<T>>) -> anyhow::Result<()> {
        todo!("insert atomically and reject duplicate Modal IDs")
    }
    pub fn query<T: Send + Sync + 'static>(&self, id: &str) -> anyhow::Result<RegisteredModal<T>> {
        todo!("downcast internally and return a correctly typed handle")
    }
    pub fn contains(&self, id: &str) -> bool {
        todo!("check definition presence")
    }
}
