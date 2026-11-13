//! Markdown Text Display. Each builder call adds a distinct component.
use crate::page::value::{IntoValue, Value};

pub struct Text<T> {
    pub content: Value<String, T>,
}
impl<T> Text<T> {
    pub fn new<V: IntoValue<String, T>>(content: V) -> Self {
        Self { content: content.into_value() }
    }
}
