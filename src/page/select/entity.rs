//! Typed User, Role, Mentionable, and Channel select menu definitions.
use std::future::Future;
use std::marker::PhantomData;
use crate::app::{Ctx,Result};
use crate::page::execution::{Execution,SelectedChannel,SelectedMentionable,SelectedRole,SelectedUser};
use crate::page::value::{IntoValue,Value};

pub struct EntitySelect<T,E,S> {
    pub id:String,
    pub placeholder:Option<Value<String,T,E>>,
    pub min_values:Option<u8>,
    pub max_values:Option<u8>,
    pub disabled:Value<bool,T,E>,
    pub channel_types:Vec<serenity::all::ChannelType>,
    pub(crate) execution:Execution<T,E,Vec<S>>,
    marker:PhantomData<fn()->S>,
}

impl<T,E,S> EntitySelect<T,E,S> {
    pub fn new<F,Fut>(id:impl Into<String>,callback:F)->Self
    where
        F:Fn(Ctx<E>,T,Vec<S>)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static,E:Send+Sync+'static,S:Send+'static
    {
        todo!("construct typed entity-select execution")
    }
    pub fn set_placeholder<V:IntoValue<String,T,E>>(mut self,value:V)->Self {
        self.placeholder=Some(value.into_value()); self
    }
    pub fn set_min_values(mut self,value:u8)->Self { self.min_values=Some(value); self }
    pub fn set_max_values(mut self,value:u8)->Self { self.max_values=Some(value); self }
    pub fn set_disabled<V:IntoValue<bool,T,E>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
    pub fn set_channel_types(mut self,value:impl IntoIterator<Item=serenity::all::ChannelType>)->Self {
        self.channel_types=value.into_iter().collect(); self
    }
}

pub type UserSelect<T,E> = EntitySelect<T,E,SelectedUser>;
pub type RoleSelect<T,E> = EntitySelect<T,E,SelectedRole>;
pub type MentionableSelect<T,E> = EntitySelect<T,E,SelectedMentionable>;
pub type ChannelSelect<T,E> = EntitySelect<T,E,SelectedChannel>;

use super::SelectMenu;
use super::string::StringSelect;
impl<T,E> From<StringSelect<T,E>> for SelectMenu<T,E> {
    fn from(value:StringSelect<T,E>)->Self { Self::String(value) }
}
impl<T,E> From<UserSelect<T,E>> for SelectMenu<T,E> {
    fn from(value:UserSelect<T,E>)->Self { Self::User(value) }
}
impl<T,E> From<RoleSelect<T,E>> for SelectMenu<T,E> {
    fn from(value:RoleSelect<T,E>)->Self { Self::Role(value) }
}
impl<T,E> From<MentionableSelect<T,E>> for SelectMenu<T,E> {
    fn from(value:MentionableSelect<T,E>)->Self { Self::Mentionable(value) }
}
impl<T,E> From<ChannelSelect<T,E>> for SelectMenu<T,E> {
    fn from(value:ChannelSelect<T,E>)->Self { Self::Channel(value) }
}
