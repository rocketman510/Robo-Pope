//! Thumbnail may be used as a Section accessory, never as a standalone child.
use crate::page::value::{IntoValue,Value};

pub struct Thumbnail<T,E> {
    pub url:Value<String,T,E>,
    pub description:Option<Value<String,T,E>>,
    pub spoiler:Value<bool,T,E>,
}
impl<T,E> Thumbnail<T,E> {
    pub fn new<V:IntoValue<String,T,E>>(url:V)->Self {
        Self { url:url.into_value(), description:None, spoiler:Value::Static(false) }
    }
    pub fn set_description<V:IntoValue<String,T,E>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    pub fn set_spoiler<V:IntoValue<bool,T,E>>(mut self,value:V)->Self {
        self.spoiler=value.into_value(); self
    }
}
