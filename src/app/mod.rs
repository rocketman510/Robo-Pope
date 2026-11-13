//! App ownership, typed definition registries, persistence, and interaction routing.
pub mod context;
pub mod database;
pub mod error;
pub mod handler;
pub mod registry;
pub mod router;
pub mod store;
pub mod executions;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use serenity::all::Http;
use registry::{ModalRegistry, PageRegistry};
use router::InteractionRouter;
use store::Store;
use executions::ExecutionRegistry;

pub use anyhow::{Context, Error, Result};
pub use context::{Ctx, RenderCtx};
pub use database::Database;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Monolithic owner of framework and application state.
///
/// Registered definitions, the database, HTTP client, and interaction router
/// live here. Pages and components carry only their own serialized data type;
/// callbacks reach all application state through Ctx or RenderCtx.
pub struct App {
    /// Shared host application's database aggregate.
    pub db: Arc<Database>,
    /// Framework persistence for custom-ID execution and pagination records.
    pub(crate) store: Arc<dyn Store>,
    /// Heterogeneous registry of immutable registered Page definitions.
    pub pages: PageRegistry,
    /// Heterogeneous registry of immutable registered Modal definitions.
    pub modals: ModalRegistry,
    pub(crate) http: Arc<Http>,
    pub(crate) router: InteractionRouter,
    /// Process-local type-erased execution callbacks (never persisted).
    pub(crate) executions: ExecutionRegistry,
}
impl Clone for App {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            store: self.store.clone(),
            pages: self.pages.clone(),
            modals: self.modals.clone(),
            http: self.http.clone(),
            router: self.router.clone(),
            executions: self.executions.clone(),
        }
    }
}
impl App {
    /// Creates an App with empty definition registries and shared infrastructure.
    pub fn new(http: Arc<Http>, db: Database, store: Arc<dyn Store>) -> Self {
        Self {
            db: Arc::new(db),
            store,
            pages: PageRegistry::default(),
            modals: ModalRegistry::default(),
            http,
            router: InteractionRouter::default(),
            executions: ExecutionRegistry::default(),
        }
    }
    /// Routes an incoming Discord interaction to its registered execution.
    ///
    /// Unknown or malformed execution references are returned as errors. A
    /// valid custom ID identifies an execution but does not authorize it.
    pub async fn dispatch_interaction(&self, interaction: serenity::all::Interaction) -> anyhow::Result<()> {
        self.router.dispatch(self, interaction).await
    }
    /// Creates a render-time context tied to this App and definition ID.
    pub fn render_context(&self, definition_id: impl Into<Arc<str>>) -> RenderCtx {
        RenderCtx::new(self.clone(), definition_id.into())
    }
    /// Borrows the Store used for persisted interaction execution records.
    pub fn store(&self) -> &dyn Store { self.store.as_ref() }
}
