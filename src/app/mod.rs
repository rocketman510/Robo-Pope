//! App ownership, typed definition registries, persistence, and interaction routing.
pub mod context;
pub mod database;
pub mod error;
pub mod handler;
pub mod registry;
pub mod router;
pub mod store;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use serenity::all::Http;
use registry::{ModalRegistry,PageRegistry};
use router::InteractionRouter;
use store::Store;

pub use anyhow::{Context,Error,Result};
pub use context::{Ctx,RenderCtx};
pub use database::Database;

pub type BoxFuture<'a,T> = Pin<Box<dyn Future<Output=T>+Send+'a>>;

/// E is the host application's typed database/service bundle. T remains the
/// only serialized per-Page instance data; E is never serialized.
pub struct App<E=()> {
    pub db:Arc<Database<E>>,
    pub pages:PageRegistry<E>,
    pub modals:ModalRegistry<E>,
    pub(crate) http:Arc<Http>,
    pub(crate) router:InteractionRouter<E>,
}
impl<E> Clone for App<E> {
    fn clone(&self)->Self {
        Self { db:self.db.clone(),pages:self.pages.clone(),modals:self.modals.clone(),
            http:self.http.clone(),router:self.router.clone() }
    }
}
impl<E:Send+Sync+'static> App<E> {
    pub fn new(http:Arc<Http>,db:Database<E>)->Self {
        todo!("construct shared App state and empty registries")
    }
    pub async fn dispatch_interaction(
        &self,interaction:serenity::all::Interaction
    )->anyhow::Result<()> {
        self.router.dispatch(self,interaction).await
    }
    pub fn render_context(&self,definition_id:impl Into<Arc<str>>)->RenderCtx<E> {
        RenderCtx::new(self.clone(),definition_id.into())
    }
    pub fn store(&self)->&dyn Store { self.db.store.as_ref() }
}
