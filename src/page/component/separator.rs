//! Separator spacing and divider state are render-time properties.
use crate::page::value::{IntoValue,Value};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum SeparatorSpacing { Small, Large }
impl<T> IntoValue<SeparatorSpacing,T> for SeparatorSpacing {
    fn into_value(self)->Value<SeparatorSpacing,T> { Value::Static(self) }
}

pub struct Separator<T> {
    pub spacing:Value<SeparatorSpacing,T>,
    pub divider:Value<bool,T>,
}
impl<T> Separator<T> {
    pub fn new()->Self {
        Self { spacing:Value::Static(SeparatorSpacing::Small), divider:Value::Static(true) }
    }
    pub fn set_spacing<V:IntoValue<SeparatorSpacing,T>>(mut self,value:V)->Self {
        self.spacing=value.into_value(); self
    }
    pub fn set_divider<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.divider=value.into_value(); self
    }
}
