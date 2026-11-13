//! Typed User, Role, Mentionable, and Channel select menu definitions.
use std::future::Future;
use crate::app::{Ctx,Result};
use crate::page::execution::{Execution,SelectedChannel,SelectedMentionable,SelectedRole,SelectedUser};
use crate::page::value::{IntoValue,Value};

pub struct EntitySelect<T,S> {
    pub id:String,
    pub placeholder:Option<Value<String,T>>,
    pub min_values:Option<u8>,
    pub max_values:Option<u8>,
    pub disabled:Value<bool,T>,
    pub channel_types:Vec<serenity::all::ChannelType>,
    pub(crate) execution:Execution<T,Vec<S>>,
    
}

impl<T,S> EntitySelect<T,S> {
    pub fn new<F,Fut>(id:impl Into<String>,callback:F)->Self
    where
        F:Fn(Ctx,T,Vec<S>)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static,S:Send+'static
    {
        todo!("construct typed entity-select execution")
    }
    pub fn set_placeholder<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.placeholder=Some(value.into_value()); self
    }
    pub fn set_min_values(mut self,value:u8)->Self { self.min_values=Some(value); self }
    pub fn set_max_values(mut self,value:u8)->Self { self.max_values=Some(value); self }
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
    pub fn set_channel_types(mut self,value:impl IntoIterator<Item=serenity::all::ChannelType>)->Self {
        self.channel_types=value.into_iter().collect(); self
    }
}

pub type UserSelect<T> = EntitySelect<T,SelectedUser>;
pub type RoleSelect<T> = EntitySelect<T,SelectedRole>;
pub type MentionableSelect<T> = EntitySelect<T,SelectedMentionable>;
pub type ChannelSelect<T> = EntitySelect<T,SelectedChannel>;

use super::SelectMenu;
use super::string::StringSelect;
impl<T> From<StringSelect<T>> for SelectMenu<T> {
    fn from(value:StringSelect<T>)->Self { Self::String(value) }
}
impl<T> From<UserSelect<T>> for SelectMenu<T> {
    fn from(value:UserSelect<T>)->Self { Self::User(value) }
}
impl<T> From<RoleSelect<T>> for SelectMenu<T> {
    fn from(value:RoleSelect<T>)->Self { Self::Role(value) }
}
impl<T> From<MentionableSelect<T>> for SelectMenu<T> {
    fn from(value:MentionableSelect<T>)->Self { Self::Mentionable(value) }
}
impl<T> From<ChannelSelect<T>> for SelectMenu<T> {
    fn from(value:ChannelSelect<T>)->Self { Self::Channel(value) }
}
