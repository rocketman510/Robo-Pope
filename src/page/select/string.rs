//! String Select options and their typed execution payload.
use std::future::Future;
use crate::app::{Ctx, Result};
use crate::page::execution::Execution;
use crate::page::value::{IntoValue, Value};

/// One String Select option with a stable submitted value and render-time display properties.
pub struct SelectOption<T, D = std::sync::Arc<()>> {
    /// Option label shown in the menu.
    pub label: Value<String, T, D>,
    /// Stable value returned to the callback when this option is selected.
    pub value: String,
    /// Optional option description shown in the menu.
    pub description: Option<Value<String, T, D>>,
    /// Optional emoji shown beside the option label.
    pub emoji: Option<Value<Option<String>, T, D>>,
    /// Whether the option can be selected.
    pub disabled: Value<bool, T, D>,
}

impl<T, D> SelectOption<T, D> {
    /// Creates an enabled option with a developer-defined submitted value.
    pub fn new<L: IntoValue<String, T, D>>(label: L, value: impl Into<String>) -> Self {
        Self {
            label: label.into_value(),
            value: value.into(),
            description: None,
            emoji: None,
            disabled: Value::Static(false),
        }
    }

    /// Sets an optional static or dynamic option description.
    pub fn set_description<V: IntoValue<String, T, D>>(mut self, value: V) -> Self {
        self.description = Some(value.into_value());
        self
    }

    /// Sets an optional static or dynamic option emoji.
    pub fn set_emoji<V: IntoValue<Option<String>, T, D>>(mut self, value: V) -> Self {
        self.emoji = Some(value.into_value());
        self
    }

    /// Sets a static or dynamic disabled state for this option.
    pub fn set_disabled<V: IntoValue<bool, T, D>>(mut self, value: V) -> Self {
        self.disabled = value.into_value();
        self
    }
}

/// String Select menu. Its execution receives the submitted option values as strings.
pub struct StringSelect<T, D = std::sync::Arc<()>> {
    /// Stable developer-defined execution ID within the containing Page.
    pub id: String,
    /// Optional placeholder displayed before a selection is made.
    pub placeholder: Option<Value<String, T, D>>,
    /// Available options; Discord permits at most twenty-five.
    pub options: Vec<SelectOption<T, D>>,
    /// Minimum number of options that must be selected, when set.
    pub min_values: Option<u8>,
    /// Maximum number of options that may be selected, when set.
    pub max_values: Option<u8>,
    /// Whether the menu is disabled after render-time resolution.
    pub disabled: Value<bool, T, D>,
    pub(crate) execution: Execution<T, Vec<String>, D>,
}

impl<T, D> StringSelect<T, D> {
    /// Creates a menu with a placeholder, stable execution ID, options, and callback.
    /// The callback receives all selected option values in selection order.
    pub fn new<F, Fut>(
        placeholder: impl IntoValue<String, T, D>,
        id: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption<T, D>>,
        callback: F,
    ) -> Self
    where
        F: Fn(Ctx<D>, T, Vec<String>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static,
        D: Send + Sync + 'static,
    {
        let id = id.into();
        Self {
            id: id.clone(),
            placeholder: Some(placeholder.into_value()),
            options: options.into_iter().collect(),
            min_values: None,
            max_values: None,
            disabled: Value::Static(false),
            execution: Execution::with_input(id, callback),
        }
    }

    /// Sets the minimum number of options a user must select.
    pub fn set_min_values(mut self, value: u8) -> Self { self.min_values = Some(value); self }

    /// Sets the maximum number of options a user may select.
    pub fn set_max_values(mut self, value: u8) -> Self { self.max_values = Some(value); self }

    /// Sets a static or dynamic disabled state for this menu.
    pub fn set_disabled<V: IntoValue<bool, T, D>>(mut self, value: V) -> Self {
        self.disabled = value.into_value();
        self
    }
}
