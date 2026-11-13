//! Type-erased host application database, independent from framework persistence.
use std::any::Any;
use std::sync::Arc;

/// Type-erased handle to the host application's concrete database and repositories.
#[derive(Clone)]
pub struct Database { value: Arc<dyn Any + Send + Sync> }
impl Database {
    /// Stores any host database aggregate that is safe to share across async tasks.
    pub fn new<T: Any + Send + Sync>(value: T) -> Self { Self { value: Arc::new(value) } }

    /// Returns the host database as its original concrete type, or an error on mismatch.
    pub fn get<T: Any + Send + Sync>(&self) -> anyhow::Result<&T> {
        self.value.downcast_ref::<T>().ok_or_else(|| {
            anyhow::anyhow!("The App database does not contain the requested type")
        })
    }
}
