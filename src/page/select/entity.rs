//! Typed User, Role, Mentionable, and Channel select menu definitions.
use std::future::Future;
use crate::app::{Ctx,Result};
use crate::page::execution::{Execution,SelectedChannel,SelectedMentionable,SelectedRole,SelectedUser};
use crate::page::value::{IntoValue,Value};

/// Generic entity-selection menu. The type parameter `S` is the resolved
/// selected entity passed to its execution callback.
pub struct EntitySelect<T,S> {
    /// Stable developer-defined execution ID within the containing Page.
    /// Stable execution ID within the containing Page.
    pub id:String,
    /// Optional placeholder displayed before a selection is made.
    pub placeholder:Option<Value<String,T>>,
    /// Minimum number of entities that must be selected, when set.
    pub min_values:Option<u8>,
    /// Maximum number of entities that may be selected, when set.
    pub max_values:Option<u8>,
    /// Whether the menu is disabled after render-time resolution.
    pub disabled:Value<bool,T>,
    /// Allowed Channel types; used by Channel Select menus.
    pub channel_types:Vec<serenity::all::ChannelType>,
    pub(crate) execution:Execution<T,Vec<S>>,
    
}

impl<T,S> EntitySelect<T,S> {
    /// Creates an entity-select menu with a callback receiving selected entities.
    pub fn new<F,Fut>(id:impl Into<String>,callback:F)->Self
    where
        F:Fn(Ctx,T,Vec<S>)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static,S:Send+'static
    {
        todo!("construct typed entity-select execution")
    }
    /// Sets an optional static or dynamic placeholder.
    pub fn set_placeholder<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.placeholder=Some(value.into_value()); self
    }
    /// Sets the minimum number of entities a user must select.
    pub fn set_min_values(mut self,value:u8)->Self { self.min_values=Some(value); self }
    /// Sets the maximum number of entities a user may select.
    pub fn set_max_values(mut self,value:u8)->Self { self.max_values=Some(value); self }
    /// Sets a static or dynamic disabled state.
    pub fn set_disabled<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.disabled=value.into_value(); self
    }
    /// Restricts Channel Select results to the supplied Discord channel types.
    pub fn set_channel_types(mut self,value:impl IntoIterator<Item=serenity::all::ChannelType>)->Self {
        self.channel_types=value.into_iter().collect(); self
    }
}

/// Select menu that returns selected Users.
pub type UserSelect<T> = EntitySelect<T,SelectedUser>;
/// Select menu that returns selected Roles.
pub type RoleSelect<T> = EntitySelect<T,SelectedRole>;
/// Select menu that returns selected Users or Roles.
pub type MentionableSelect<T> = EntitySelect<T,SelectedMentionable>;
/// Select menu that returns selected Channels.
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
