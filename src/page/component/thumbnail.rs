//! Thumbnail may be used as a Section accessory, never as a standalone child.
use crate::page::value::{IntoValue,Value};

/// Thumbnail accessory for a Section. It cannot be inserted as a standalone component.
pub struct Thumbnail<T> {
    /// Image URL resolved at render time.
    pub url:Value<String,T>,
    /// Optional image description resolved at render time.
    pub description:Option<Value<String,T>>,
    /// Whether Discord marks the thumbnail as a spoiler.
    pub spoiler:Value<bool,T>,
}
impl<T> Thumbnail<T> {
    /// Creates a thumbnail with the given static or dynamic URL.
    pub fn new<V:IntoValue<String,T>>(url:V)->Self {
        Self { url:url.into_value(), description:None, spoiler:Value::Static(false) }
    }
    /// Sets an optional static or dynamic thumbnail description.
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    /// Sets the thumbnail's static or dynamic spoiler state.
    pub fn set_spoiler<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.spoiler=value.into_value(); self
    }
}
