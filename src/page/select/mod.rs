//! One Page API for all supported Discord selection menu kinds.
pub mod entity;
pub mod string;

pub use entity::{ChannelSelect, EntitySelect, MentionableSelect, RoleSelect, UserSelect};
pub use string::{SelectOption, StringSelect};

pub enum SelectMenu<T> {
    String(StringSelect<T>),
    User(UserSelect<T>),
    Role(RoleSelect<T>),
    Mentionable(MentionableSelect<T>),
    Channel(ChannelSelect<T>),
}
