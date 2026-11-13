//! Markdown Text Display. Each builder call adds a distinct component.
use crate::page::value::{IntoValue, Value};

/// Text Display containing Markdown text. The content is resolved when the Page renders.
pub struct Text<T> {
    /// Markdown content resolved when the Page renders.
    pub content: Value<String, T>,
}
impl<T> Text<T> {
    /// Creates a Text Display from a static string or dynamic callback.
    pub fn new<V: IntoValue<String, T>>(content: V) -> Self {
        Self { content: content.into_value() }
    }
}
