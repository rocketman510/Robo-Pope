//! A Link Button opens a URL. It never creates an execution or Store record.
use crate::page::value::{IntoValue,Value};

/// Button that opens a URL. It is not an interaction execution and creates
/// no Store record or callback.
pub struct LinkButton<T> {
    /// Label shown to the user after render-time resolution.
    pub label:Value<String,T>,
    /// Destination URL opened by Discord.
    pub url:Value<String,T>,
    /// Whether the rendered Link Button is disabled.
    pub disabled:Value<bool,T>,
}
impl<T> LinkButton<T> {
    /// Creates a URL button with static or dynamic label and URL properties.
    pub fn new<L:IntoValue<String,T>,U:IntoValue<String,T>>(label:L,url:U)->Self {
        todo!("construct the non-interactive URL button")
    }
    /// Sets a static or dynamic disabled state.
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}
