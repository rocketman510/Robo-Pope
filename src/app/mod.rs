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
use registry::{ModalRegistry, PageRegistry};
use router::InteractionRouter;
use store::Store;

pub use anyhow::{Context, Error, Result};
pub use context::{Ctx, RenderCtx};
pub use database::Database;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Monolithic owner of framework and host-application state.
/// Page data remains the only generic, serialized per-instance state.
pub struct App {
    pub db: Arc<Database>,
    pub pages: PageRegistry,
    pub modals: ModalRegistry,
    pub(crate) http: Arc<Http>,
    pub(crate) router: InteractionRouter,
}
impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            pages: self.pages.clone(),
            modals: self.modals.clone(),
            http: self.http.clone(),
            router: self.router.clone(),
        }
    }
}
impl App {
    pub fn new(http: Arc<Http>, db: Database) -> Self {
        todo!("construct shared App state and empty registries")
    }
    pub async fn dispatch_interaction(&self, interaction: serenity::all::Interaction) -> anyhow::Result<()> {
        self.router.dispatch(self, interaction).await
    }
    pub fn render_context(&self, definition_id: impl Into<Arc<str>>) -> RenderCtx {
        RenderCtx::new(self.clone(), definition_id.into())
    }
    pub fn store(&self) -> &dyn Store { self.db.store.as_ref() }
}
