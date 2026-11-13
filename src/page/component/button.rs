//! Interactive Button with display properties and an explicit execution.
use crate::app::{Ctx, Result};
use crate::page::execution::Execution;
use crate::page::value::{IntoValue, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
}

impl<T, E> IntoValue<ButtonStyle, T, E> for ButtonStyle {
    fn into_value(self) -> Value<ButtonStyle, T, E> { Value::Static(self) }
}

pub struct Button<T, E> {
    pub id: String,
    pub label: Value<String, T, E>,
    pub style: Value<ButtonStyle, T, E>,
    pub disabled: Value<bool, T, E>,
    pub(crate) execution: Execution<T, E, ()>,
}

impl<T, E> Button<T, E> {
    pub fn new<L, S, F, Fut>(
        label: L,
        id: impl Into<String>,
        style: S,
        callback: F,
    ) -> Self
    where
        L: IntoValue<String, T, E>,
        S: IntoValue<ButtonStyle, T, E>,
        F: Fn(Ctx<E>, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static,
        E: Send + Sync + 'static,
    {
        todo!("create the explicit execution and initialize enabled state")
    }

    pub fn set_disabled<V: IntoValue<bool, T, E>>(mut self, disabled: V) -> Self {
        self.disabled = disabled.into_value();
        self
    }
}

use std::future::Future;
