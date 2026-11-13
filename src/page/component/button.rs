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

impl<T> IntoValue<ButtonStyle, T> for ButtonStyle {
    fn into_value(self) -> Value<ButtonStyle, T> { Value::Static(self) }
}

pub struct Button<T> {
    pub id: String,
    pub label: Value<String, T>,
    pub style: Value<ButtonStyle, T>,
    pub disabled: Value<bool, T>,
    pub(crate) execution: Execution<T>,
}

impl<T> Button<T> {
    pub fn new<L, S, F, Fut>(
        label: L,
        id: impl Into<String>,
        style: S,
        callback: F,
    ) -> Self
    where
        L: IntoValue<String, T>,
        S: IntoValue<ButtonStyle, T>,
        F: Fn(Ctx, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static
{
        todo!("create the explicit execution and initialize enabled state")
    }

    pub fn set_disabled<V: IntoValue<bool, T>>(mut self, disabled: V) -> Self {
        self.disabled = disabled.into_value();
        self
    }
}

use std::future::Future;
