//! A Link Button opens a URL. It never creates an execution or Store record.
use crate::page::value::{IntoValue,Value};

pub struct LinkButton<T,E> {
    pub label:Value<String,T,E>,
    pub url:Value<String,T,E>,
    pub disabled:Value<bool,T,E>,
}
impl<T,E> LinkButton<T,E> {
    pub fn new<L:IntoValue<String,T,E>,U:IntoValue<String,T,E>>(label:L,url:U)->Self {
        todo!("construct the non-interactive URL button")
    }
    pub fn set_disabled<V:IntoValue<bool,T,E>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}
