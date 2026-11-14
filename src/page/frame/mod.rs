//! Dynamic, heterogeneous region resolved as part of a Page render.
pub mod element;
pub use element::Element;

use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture, RenderCtx};

/// Dynamic region that returns heterogeneous elements at its declared Page position.
pub struct Frame<T, D = Arc<()>> {
    pub(crate) resolver: Arc<
        dyn Fn(RenderCtx<D>, Arc<T>) -> BoxFuture<'static, anyhow::Result<Vec<Element<T, D>>>>
            + Send + Sync,
    >,
}

impl<T, D> Frame<T, D>
where
    T: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    /// Creates a Frame from an asynchronous resolver.
    pub fn new<F, Fut>(resolver: F) -> Self
    where
        F: Fn(RenderCtx<D>, Arc<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<Vec<Element<T, D>>>> + Send + 'static,
    {
        Self { resolver: Arc::new(move |ctx, data| Box::pin(resolver(ctx, data))) }
    }
}
