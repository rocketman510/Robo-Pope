//! One Page API for all supported Discord selection menu kinds.
pub mod entity;
pub mod string;

pub use entity::{ChannelSelect, EntitySelect, MentionableSelect, RoleSelect, UserSelect};
pub use string::{SelectOption, StringSelect};

/// Unified menu type accepted by Page and Container builders.
/// Each variant retains its own typed selection callback.
pub enum SelectMenu<T, D = std::sync::Arc<()>> {
    String(StringSelect<T, D>),
    User(UserSelect<T, D>),
    Role(RoleSelect<T, D>),
    Mentionable(MentionableSelect<T, D>),
    Channel(ChannelSelect<T, D>),
}
