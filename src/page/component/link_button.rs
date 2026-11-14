//! A Link Button opens a URL. It never creates an execution or Store record.
use crate::page::value::{IntoValue, Value};

/// Button that opens a URL. It is not an interaction execution and creates
/// no Store record or callback.
pub struct LinkButton<T, D = std::sync::Arc<()>> {
    /// Label shown to the user after render-time resolution.
    pub label: Value<String, T, D>,
    /// Destination URL opened by Discord.
    pub url: Value<String, T, D>,
    /// Optional Unicode emoji shown beside the Link Button label.
    pub emoji: Value<Option<String>, T, D>,
    /// Whether the rendered Link Button is disabled.
    pub disabled: Value<bool, T, D>,
}

impl<T, D> LinkButton<T, D> {
    /// Creates a URL Button with static or dynamic display properties.
    pub fn new<L, U, E, B>(label: L, url: U, emoji: E, disabled: B) -> Self
    where
        L: IntoValue<String, T, D>,
        U: IntoValue<String, T, D>,
        E: IntoValue<Option<String>, T, D>,
        B: IntoValue<bool, T, D>,
    {
        Self {
            label: label.into_value(),
            url: url.into_value(),
            emoji: emoji.into_value(),
            disabled: disabled.into_value(),
        }
    }

    /// Sets a static or dynamic optional emoji.
    pub fn set_emoji<V: IntoValue<Option<String>, T, D>>(mut self, emoji: V) -> Self {
        self.emoji = emoji.into_value();
        self
    }

    /// Sets a static or dynamic disabled state.
    pub fn set_disabled<V: IntoValue<bool, T, D>>(mut self, value: V) -> Self {
        self.disabled = value.into_value();
        self
    }
}
