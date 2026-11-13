//! Selection menus.
//!
//! Page has one add_selection_menu method. The concrete menu determines the
//! Discord kind: String, User, Role, Mentionable, or Channel. Shared
//! abstraction keeps Page composition uniform without flattening all entity
//! selections into strings.
//!
//! Placeholder, labels, descriptions, and disabled state follow the
//! static-or-dynamic Value contract where appropriate. Selected values are
//! interaction data supplied to the explicit callback separately from Page
//! data.
//!
//! TODO: Define SelectMenu<T, E>, SelectOption<T, E>, shared settings, and
//!       conversions from each concrete menu type.
pub mod entity;
pub mod string;
