//! String Select options and their typed execution payload.
use std::future::Future;
use crate::app::{Ctx, Result};
use crate::page::execution::Execution;
use crate::page::value::{IntoValue,Value};

pub struct SelectOption<T> {
    pub label:Value<String,T>,
    pub value:String,
    pub description:Option<Value<String,T>>,
    pub disabled:Value<bool,T>,
}
impl<T> SelectOption<T> {
    pub fn new<L:IntoValue<String,T>>(label:L,value:impl Into<String>)->Self {
        todo!("create option with its stable submitted value and default enabled state")
    }
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}

pub struct StringSelect<T> {
    pub id:String,
    pub placeholder:Option<Value<String,T>>,
    pub options:Vec<SelectOption<T>>,
    pub min_values:Option<u8>,
    pub max_values:Option<u8>,
    pub disabled:Value<bool,T>,
    pub(crate) execution:Execution<T,Vec<String>>,
}
impl<T> StringSelect<T> {
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
    pub fn set_min_values(mut self,value:u8)->Self { self.min_values=Some(value); self }
    pub fn set_max_values(mut self,value:u8)->Self { self.max_values=Some(value); self }
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
}
