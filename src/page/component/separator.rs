//! Separator spacing and divider state are render-time properties.
use crate::page::value::{IntoValue,Value};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SeparatorSpacing { Small, Large }
impl<T,E> IntoValue<SeparatorSpacing,T,E> for SeparatorSpacing {
    fn into_value(self)->Value<SeparatorSpacing,T,E> { Value::Static(self) }
}

pub struct Separator<T,E> {
    pub spacing:Value<SeparatorSpacing,T,E>,
    pub divider:Value<bool,T,E>,
}
impl<T,E> Separator<T,E> {
    pub fn new()->Self {
        Self { spacing:Value::Static(SeparatorSpacing::Small), divider:Value::Static(true) }
    }
    pub fn set_spacing<V:IntoValue<SeparatorSpacing,T,E>>(mut self,value:V)->Self {
        self.spacing=value.into_value(); self
    }
    pub fn set_divider<V:IntoValue<bool,T,E>>(mut self,value:V)->Self {
        self.divider=value.into_value(); self
    }
}
