//! A Link Button opens a URL. It never creates an execution or Store record.
use crate::page::value::{IntoValue,Value};

pub struct LinkButton<T> {
    pub label:Value<String,T>,
    pub url:Value<String,T>,
    pub disabled:Value<bool,T>,
}
impl<T> LinkButton<T> {
    pub fn new<L:IntoValue<String,T>,U:IntoValue<String,T>>(label:L,url:U)->Self {
        todo!("construct the non-interactive URL button")
    }
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}
