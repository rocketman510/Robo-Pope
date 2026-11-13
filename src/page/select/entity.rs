//! User, Role, Mentionable, and Channel Select menus.
//!
//! Each entity menu shares placeholder, selection bounds, disabled state,
//! and execution identity, but its callback receives the appropriate
//! Discord entity type. Include resolved Discord records when supplied,
//! alongside stable entity IDs.
//!
//! Page still exposes one add_selection_menu method. Internally each menu
//! maps to its correct Serenity builder and typed Selection payload.
//!
//! TODO: Define the four entity menu types and choice structs.
//! TODO: Support defaults and channel type filters where Discord permits.
//! TODO: Never parse malformed IDs into fabricated zero-valued entities.
