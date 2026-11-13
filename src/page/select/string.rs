//! String Select options and their typed execution payload.
use std::future::Future;
use crate::app::{Ctx, Result};
use crate::page::execution::Execution;
use crate::page::value::{IntoValue,Value};

/// One String Select option with a stable submitted value and render-time display properties.
pub struct SelectOption<T> {
    /// Option label shown in the menu.
    pub label:Value<String,T>,
    /// Stable value returned to the callback when this option is selected.
    pub value:String,
    /// Optional option description shown in the menu.
    pub description:Option<Value<String,T>>,
    /// Whether the option can be selected.
    pub disabled:Value<bool,T>,
}
impl<T> SelectOption<T> {
    /// Creates an enabled option with a developer-defined submitted value.
    pub fn new<L:IntoValue<String,T>>(label:L,value:impl Into<String>)->Self {
        todo!("create option with its stable submitted value and default enabled state")
    }
    /// Sets an optional static or dynamic option description.
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    /// Sets a static or dynamic disabled state for this option.
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}

/// String Select menu. Its execution receives the submitted option values as strings.
pub struct StringSelect<T> {
    /// Stable developer-defined execution ID within the containing Page.
    pub id:String,
    /// Optional placeholder displayed before a selection is made.
    pub placeholder:Option<Value<String,T>>,
    /// Available options; Discord permits at most twenty-five.
    pub options:Vec<SelectOption<T>>,
    /// Minimum number of options that must be selected, when set.
    pub min_values:Option<u8>,
    /// Maximum number of options that may be selected, when set.
    pub max_values:Option<u8>,
    /// Whether the menu is disabled after render-time resolution.
    pub disabled:Value<bool,T>,
    pub(crate) execution:Execution<T,Vec<String>>,
}
impl<T> StringSelect<T> {
    /// Creates a menu with a placeholder, stable execution ID, options, and callback.
    /// The callback receives all selected option values in selection order.
    pub fn new<F,Fut>(
        placeholder:impl IntoValue<String,T>,
        id:impl Into<String>,
        options:impl IntoIterator<Item=SelectOption<T>>,
        callback:F,
    )->Self
    where
        F:Fn(Ctx,T,Vec<String>)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static,{
        todo!("construct the menu and explicit selection execution")
    }
    /// Sets the minimum number of options a user must select.
    pub fn set_min_values(mut self,value:u8)->Self { self.min_values=Some(value); self }
    /// Sets the maximum number of options a user may select.
    pub fn set_max_values(mut self,value:u8)->Self { self.max_values=Some(value); self }
    /// Sets a static or dynamic disabled state for this menu.
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}
