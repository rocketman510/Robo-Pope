//! Heterogeneous registries; concrete T is retained by typed query handles.
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::page::{PageDefinition, RegisteredPage};
use crate::page::modal::{ModalDefinition, RegisteredModal};

pub struct PageRegistry<E> {
    entries: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    marker: std::marker::PhantomData<fn() -> E>,
}
impl<E> Clone for PageRegistry<E> {
    fn clone(&self) -> Self {
        Self { entries: self.entries.clone(), marker: std::marker::PhantomData }
    }
}
impl<E: Send + Sync + 'static> Default for PageRegistry<E> {
    fn default() -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())), marker: std::marker::PhantomData }
    }
}
impl<E: Send + Sync + 'static> PageRegistry<E> {
    pub fn insert<T: Send + Sync + 'static>(
        &self,
        definition: Arc<PageDefinition<T, E>>,
    ) -> anyhow::Result<()> {
        todo!("insert atomically and reject duplicate Page IDs")
    }
    pub fn query<T: Send + Sync + 'static>(
        &self,
        id: &str,
    ) -> anyhow::Result<RegisteredPage<T, E>> {
        todo!("downcast internally and return a correctly typed handle")
    }
    pub fn contains(&self, id: &str) -> bool {
        todo!("check definition presence")
    }
}

pub struct ModalRegistry<E> {
    entries: Arc<RwLock<HashMap<String, Arc<dyn Any + Send + Sync>>>>,
    marker: std::marker::PhantomData<fn() -> E>,
}
impl<E> Clone for ModalRegistry<E> {
    fn clone(&self) -> Self {
        Self { entries: self.entries.clone(), marker: std::marker::PhantomData }
    }
}
impl<E: Send + Sync + 'static> Default for ModalRegistry<E> {
    fn default() -> Self {
        Self { entries: Arc::new(RwLock::new(HashMap::new())), marker: std::marker::PhantomData }
    }
}
impl<E: Send + Sync + 'static> ModalRegistry<E> {
    pub fn insert<T: Send + Sync + 'static>(
        &self,
        definition: Arc<ModalDefinition<T, E>>,
    ) -> anyhow::Result<()> {
        todo!("insert atomically and reject duplicate Modal IDs")
    }
    pub fn query<T: Send + Sync + 'static>(
        &self,
        id: &str,
    ) -> anyhow::Result<RegisteredModal<T, E>> {
        todo!("downcast internally and return a correctly typed handle")
    }
    pub fn contains(&self, id: &str) -> bool {
        todo!("check definition presence")
    }
}
