//! Markdown Text Display. Each builder call adds a distinct component.
use crate::page::value::{IntoValue, Value};

pub struct Text<T, E> {
    pub content: Value<String, T, E>,
}
impl<T, E> Text<T, E> {
    pub fn new<V: IntoValue<String, T, E>>(content: V) -> Self {
        Self { content: content.into_value() }
    }
}
