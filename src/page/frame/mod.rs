//! Dynamic, heterogeneous region with optional automatic pagination.
pub mod element;
pub mod pagination;

pub use element::{Element, FrameIdentity};

use std::future::Future;
use std::sync::Arc;

use crate::app::{BoxFuture, RenderCtx};

/// Dynamic region that returns heterogeneous elements at render time.
///
/// The resolver receives the concrete App context and shared Page data. When
/// pagination is enabled, the framework splits only between complete elements.
pub struct Frame<T, D = Arc<()>> {
    /// Stable developer-defined ID for this Frame within its Page.
    pub id: String,
    pub(crate) resolver: Arc<
        dyn Fn(RenderCtx<D>, Arc<T>) -> BoxFuture<'static, anyhow::Result<Vec<Element<T, D>>>>
            + Send
            + Sync,
    >,
    /// Whether the Frame's resolved elements are automatically paginated.
    pub paginate: bool,
}

impl<T, D> Frame<T, D>
where
    T: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    /// Creates a Frame with a stable ID and an asynchronous element resolver.
    pub fn new<F, Fut>(id: impl Into<String>, resolver: F) -> Self
    where
        F: Fn(RenderCtx<D>, Arc<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<Vec<Element<T, D>>>> + Send + 'static,
    {
        Self {
            id: id.into(),
            resolver: Arc::new(move |ctx, data| Box::pin(resolver(ctx, data))),
            paginate: false,
        }
    }

    /// Enables automatic pagination for this Frame.
    pub fn paginate(mut self) -> Self {
        self.paginate = true;
        self
    }
}
