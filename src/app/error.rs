//! The framework uses anyhow throughout. There is intentionally no
//! framework-specific error enum or custom Result alias.
pub use anyhow::{Context, Error, Result};
