//! Typed User, Role, Mentionable, and Channel select menu definitions.
use std::future::Future;

use crate::app::{Ctx, Result};
use crate::page::execution::{
    Execution, SelectedChannel, SelectedMentionable, SelectedRole, SelectedUser,
};
use crate::page::value::{IntoValue, Value};

/// Generic entity-selection menu. S is the resolved selected entity passed to its callback.
pub struct EntitySelect<T, S, D = std::sync::Arc<()>> {
    /// Stable developer-defined execution ID within the containing Page.
    pub id: String,
    /// Optional placeholder displayed before a selection is made.
    pub placeholder: Option<Value<String, T, D>>,
    /// Minimum number of entities that must be selected, when set.
    pub min_values: Option<u8>,
    /// Maximum number of entities that may be selected, when set.
    pub max_values: Option<u8>,
    /// Whether the menu is disabled after render-time resolution.
    pub disabled: Value<bool, T, D>,
    /// Allowed Channel types; used by Channel Select menus.
    pub channel_types: Vec<serenity::all::ChannelType>,
    pub(crate) execution: Execution<T, Vec<S>, D>,
}

impl<T, S, D> EntitySelect<T, S, D> {
    /// Creates an entity-select menu with a callback receiving selected entities.
    pub fn new<F, Fut>(id: impl Into<String>, callback: F) -> Self
    where
        F: Fn(Ctx<D>, T, Vec<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static,
        S: Send + 'static,
        D: Send + Sync + 'static,
    {
        let id = id.into();
        Self {
            id: id.clone(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: Value::Static(false),
            channel_types: Vec::new(),
            execution: Execution::with_input(id, callback),
        }
    }

    /// Sets an optional static or dynamic placeholder.
    pub fn set_placeholder<V: IntoValue<String, T, D>>(mut self, value: V) -> Self {
        self.placeholder = Some(value.into_value());
        self
    }

    /// Sets the minimum number of entities a user must select.
    pub fn set_min_values(mut self, value: u8) -> Self { self.min_values = Some(value); self }

    /// Sets the maximum number of entities a user may select.
    pub fn set_max_values(mut self, value: u8) -> Self { self.max_values = Some(value); self }

    /// Sets a static or dynamic disabled state.
    pub fn set_disabled<V: IntoValue<bool, T, D>>(mut self, value: V) -> Self {
        self.disabled = value.into_value();
        self
    }

    /// Restricts Channel Select results to the supplied Discord channel types.
    pub fn set_channel_types(mut self, value: impl IntoIterator<Item = serenity::all::ChannelType>) -> Self {
        self.channel_types = value.into_iter().collect();
        self
    }
}

/// Select menu that returns selected Users.
pub type UserSelect<T, D = std::sync::Arc<()>> = EntitySelect<T, SelectedUser, D>;
/// Select menu that returns selected Roles.
pub type RoleSelect<T, D = std::sync::Arc<()>> = EntitySelect<T, SelectedRole, D>;
/// Select menu that returns selected Users or Roles.
pub type MentionableSelect<T, D = std::sync::Arc<()>> = EntitySelect<T, SelectedMentionable, D>;
/// Select menu that returns selected Channels.
pub type ChannelSelect<T, D = std::sync::Arc<()>> = EntitySelect<T, SelectedChannel, D>;

use super::SelectMenu;
use super::string::StringSelect;

impl<T, D> From<StringSelect<T, D>> for SelectMenu<T, D> {
    fn from(value: StringSelect<T, D>) -> Self { Self::String(value) }
}

impl<T, D> From<UserSelect<T, D>> for SelectMenu<T, D> {
    fn from(value: UserSelect<T, D>) -> Self { Self::User(value) }
}

impl<T, D> From<RoleSelect<T, D>> for SelectMenu<T, D> {
    fn from(value: RoleSelect<T, D>) -> Self { Self::Role(value) }
}

impl<T, D> From<MentionableSelect<T, D>> for SelectMenu<T, D> {
    fn from(value: MentionableSelect<T, D>) -> Self { Self::Mentionable(value) }
}

impl<T, D> From<ChannelSelect<T, D>> for SelectMenu<T, D> {
    fn from(value: ChannelSelect<T, D>) -> Self { Self::Channel(value) }
}
