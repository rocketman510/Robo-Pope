//! Thumbnail may be used as a Section accessory, never as a standalone child.
use crate::page::value::{IntoValue,Value};

pub struct Thumbnail<T> {
    pub url:Value<String,T>,
    pub description:Option<Value<String,T>>,
    pub spoiler:Value<bool,T>,
}
impl<T> Thumbnail<T> {
    pub fn new<V:IntoValue<String,T>>(url:V)->Self {
        Self { url:url.into_value(), description:None, spoiler:Value::Static(false) }
    }
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    pub fn set_spoiler<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.spoiler=value.into_value(); self
    }
}
