//! Interactive Button with display properties and an explicit execution.
use std::future::Future;
use crate::app::{Ctx, Result};
use crate::page::execution::Execution;
use crate::page::value::{IntoValue, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Discord Button style. Link buttons use their own type and do not have executions.
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
}

impl<T> IntoValue<ButtonStyle, T, D> for ButtonStyle {
    fn into_value(self) -> Value<ButtonStyle, T, D> { Value::Static(self) }
}

/// Interactive Button. Its display properties may be static or dynamic, while
/// its callback is an explicit execution tied to the containing Page's data.
pub struct Button<T, D = std::sync::Arc<()>> {
    /// Stable developer-defined execution ID within the containing Page.
    pub id: String,
    /// Label shown on the Button after render-time resolution.
    pub label: Value<String, T, D>,
    /// Visual Button style after render-time resolution.
    pub style: Value<ButtonStyle, T, D>,
    /// Optional Unicode emoji shown beside the Button label.
    pub emoji: Value<Option<String>, T>,
    /// Whether the Button is disabled in the rendered message.
    pub disabled: Value<bool, T, D>,
    pub(crate) execution: Execution<T, (), D>,
}

impl<T, D> Button<T, D> {
    /// Creates a Button with its display properties and execution callback.
    pub fn new<L, S, E, B, F, Fut>(
        label: L,
        id: impl Into<String>,
        style: S,
        emoji: E,
        disabled: B,
        callback: F,
    ) -> Self
    where
        L: IntoValue<String, T, D>,
        S: IntoValue<ButtonStyle, T, D>,
        E: IntoValue<Option<String>, T>,
        B: IntoValue<bool, T, D>,
        F: Fn(Ctx<D>, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static,
    {
        let id = id.into();
        Self {
            id: id.clone(),
            label: label.into_value(),
            style: style.into_value(),
            emoji: emoji.into_value(),
            disabled: disabled.into_value(),
            execution: Execution::new(id, callback),
        }
    }

    /// Sets a static or dynamic optional emoji.
    pub fn set_emoji<V: IntoValue<Option<String>, T>>(mut self, emoji: V) -> Self {
        self.emoji = emoji.into_value();
        self
    }

    /// Sets a static or dynamic disabled state.
    pub fn set_disabled<V: IntoValue<bool, T, D>>(mut self, disabled: V) -> Self {
        self.disabled = disabled.into_value();
        self
    }
}
