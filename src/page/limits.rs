//! Discord Components V2 and Modal constraints from `spec/Page API.md`.
//!
//! These values are the limits used by validation after dynamic properties
//! and Frame contents have been resolved.
/// Maximum combined component count in a Components V2 message, including nested components.
pub const MESSAGE_MAX_COMPONENTS: usize = 40;
/// Shared message-wide Text Display character budget.
pub const MESSAGE_MAX_TEXT_DISPLAY_CHARS: usize = 4_000;
/// Minimum populated Text Display slots in a Section.
pub const SECTION_MIN_TEXT_DISPLAYS: usize = 1;
/// Maximum Text Display slots in a Section.
pub const SECTION_MAX_TEXT_DISPLAYS: usize = 3;
/// Minimum populated Button slots in a Button Action Row.
pub const BUTTON_ROW_MIN_BUTTONS: usize = 1;
/// Maximum Button slots in a Button Action Row.
pub const BUTTON_ROW_MAX_BUTTONS: usize = 5;
/// Maximum String Select options.
pub const STRING_SELECT_MAX_OPTIONS: usize = 25;
/// Maximum Media Gallery items.
pub const MEDIA_GALLERY_MAX_ITEMS: usize = 10;
/// Maximum direct children in one Container.
pub const CONTAINER_MAX_DIRECT_CHILDREN: usize = 10;
/// Maximum length of a Discord component custom ID.
pub const CUSTOM_ID_MAX_CHARS: usize = 100;
/// Maximum Modal title length.
pub const MODAL_TITLE_MAX_CHARS: usize = 45;
/// Maximum top-level Modal components.
pub const MODAL_MAX_TOP_LEVEL_COMPONENTS: usize = 5;
/// Maximum Modal input Label length.
pub const MODAL_LABEL_MAX_CHARS: usize = 45;
/// Maximum Modal input Label description length.
pub const MODAL_LABEL_DESCRIPTION_MAX_CHARS: usize = 100;
/// Maximum Text Input length.
pub const MODAL_TEXT_INPUT_MAX_CHARS: usize = 4_000;
/// Maximum options in a Modal Select.
pub const MODAL_SELECT_MAX_OPTIONS: usize = 25;

// Count nested components recursively, but keep semantic budgets separate:
// descriptions and select labels do not consume the Text Display character
// budget. Validate after resolving dynamic properties and Frame contents.
