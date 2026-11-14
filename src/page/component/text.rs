//! Markdown Text Display. Each builder call adds a distinct component.
use crate::page::value::{IntoValue, Value};

/// Text Display containing Markdown text. The content is resolved when the Page renders.
pub struct Text<T, D = std::sync::Arc<()>> {
    /// Markdown content resolved when the Page renders.
    pub content: Value<String, T, D>,
}
impl<T, D> Text<T, D> {
    /// Creates a Text Display from a static string or dynamic callback.
    pub fn new<V: IntoValue<String, T, D>>(content: V) -> Self {
        Self { content: content.into_value() }
    }
}
