//! Separator spacing and divider state are render-time properties.
use crate::page::value::{IntoValue,Value};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
/// Discord Separator spacing choices.
pub enum SeparatorSpacing { Small, Large }
impl<T> IntoValue<SeparatorSpacing,T> for SeparatorSpacing {
    fn into_value(self)->Value<SeparatorSpacing,T> { Value::Static(self) }
}

/// Separator with render-time spacing and divider properties.
pub struct Separator<T> {
    pub spacing:Value<SeparatorSpacing,T>,
    pub divider:Value<bool,T>,
}
impl<T> Separator<T> {
    /// Creates a small Separator with its divider visible.
    pub fn new()->Self {
        Self { spacing:Value::Static(SeparatorSpacing::Small), divider:Value::Static(true) }
    }
    /// Sets a static or dynamic spacing value.
    pub fn set_spacing<V:IntoValue<SeparatorSpacing,T>>(mut self,value:V)->Self {
        self.spacing=value.into_value(); self
    }
    /// Sets whether the rendered Separator displays its divider.
    pub fn set_divider<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.divider=value.into_value(); self
    }
}
