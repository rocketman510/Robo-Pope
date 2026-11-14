//! App ownership, typed host database, persistent routing, and interaction routing.
pub mod context;
pub mod database;
pub mod error;
pub mod handler;
pub mod registry;
pub mod router;
pub mod store;
pub mod executions;

use std::any::Any;
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use serenity::all::Http;
use registry::{ModalRegistry, PageRegistry};
use router::InteractionRouter;
use store::Store;
use executions::ExecutionRegistry;

pub use anyhow::{Context, Error, Result};
pub use context::{Ctx, RenderCtx};
pub use database::Database;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// App owns the host application's concrete database type and the framework's
/// separate interaction Store.
pub struct App<D = Database> {
    /// The concrete database or repository aggregate supplied by the host application.
    pub db: D,
    /// Framework persistence for custom-ID routing records.
    pub(crate) store: Arc<dyn Store>,
    pub pages: PageRegistry,
    pub modals: ModalRegistry,
    pub(crate) http: Arc<Http>,
    pub(crate) router: InteractionRouter,
    pub(crate) executions: ExecutionRegistry,
    definition_ids: Arc<Mutex<HashSet<String>>>,
}

impl<D: Clone> Clone for App<D> {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            store: self.store.clone(),
            pages: self.pages.clone(),
            modals: self.modals.clone(),
            http: self.http.clone(),
            router: self.router.clone(),
            executions: self.executions.clone(),
            definition_ids: self.definition_ids.clone(),
        }
    }
}

impl<D> App<D> {
    /// Creates an App with empty registries and the supplied host database and Store.
    pub fn new(http: Arc<Http>, db: D, store: Arc<dyn Store>) -> Self {
        Self {
            db,
            store,
            pages: PageRegistry::default(),
            modals: ModalRegistry::default(),
            http,
            router: InteractionRouter::default(),
            executions: ExecutionRegistry::default(),
            definition_ids: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Creates another App view sharing framework state but using a different DB handle.
    fn with_database<E>(&self, db: E) -> App<E> {
        App {
            db,
            store: self.store.clone(),
            pages: self.pages.clone(),
            modals: self.modals.clone(),
            http: self.http.clone(),
            router: self.router.clone(),
            executions: self.executions.clone(),
            definition_ids: self.definition_ids.clone(),
        }
    }

    /// Atomically reserves a Page or Modal ID in the shared namespace.
    pub(crate) fn reserve_definition_id(&self, id: &str) -> anyhow::Result<()> {
        let mut ids = self.definition_ids.lock()
            .map_err(|_| anyhow::anyhow!("Definition ID registry lock was poisoned"))?;
        anyhow::ensure!(ids.insert(id.to_owned()), "Page or Modal ID is already registered: {id}");
        Ok(())
    }

    /// Releases a reservation when definition registration fails.
    pub(crate) fn release_definition_id(&self, id: &str) {
        if let Ok(mut ids) = self.definition_ids.lock() { ids.remove(id); }
    }

    /// Borrows the Store used for persisted interaction execution records.
    pub fn store(&self) -> &dyn Store { self.store.as_ref() }
}

impl<D> App<D>
where
    D: Clone + Any + Send + Sync + 'static,
{
    /// Creates an erased framework context for dynamic callbacks and interactions.
    ///
    /// The typed host DB remains available on the original App. The erased view
    /// preserves it in Database::get without requiring Page or Modal data types
    /// to carry a second host-specific generic parameter.
    pub(crate) fn erased(&self) -> App<Database> {
        let database = (&self.db as &dyn Any)
            .downcast_ref::<Database>()
            .cloned()
            .unwrap_or_else(|| Database::new(self.db.clone()));
        self.with_database(database)
    }

    /// Routes an incoming Discord interaction to its registered execution.
    pub async fn dispatch_interaction(
        &self,
        interaction: serenity::all::Interaction,
    ) -> anyhow::Result<()> {
        let erased = self.erased();
        self.router.dispatch(&erased, interaction).await
    }

    /// Creates a render-time context tied to the typed App and definition ID.
    pub fn render_context(
        &self,
        definition_id: impl Into<Arc<str>>,
    ) -> RenderCtx<D> {
        RenderCtx::new(self.clone(), definition_id.into())
    }

    /// Creates the context used by framework values, whose callback ABI is DB-erased.
    pub(crate) fn render_context_erased(
        &self,
        definition_id: impl Into<Arc<str>>,
    ) -> RenderCtx<Database> {
        RenderCtx::new(self.erased(), definition_id.into())
    }
}
