//! One Page API for all supported Discord selection menu kinds.
pub mod entity;
pub mod string;

pub use entity::{ChannelSelect, EntitySelect, MentionableSelect, RoleSelect, UserSelect};
pub use string::{SelectOption, StringSelect};

pub enum SelectMenu<T,E> {
    String(StringSelect<T,E>),
    User(UserSelect<T,E>),
    Role(RoleSelect<T,E>),
    Mentionable(MentionableSelect<T,E>),
    Channel(ChannelSelect<T,E>),
}
