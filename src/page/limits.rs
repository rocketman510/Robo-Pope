//! Discord Components V2 and Modal constraints.
//!
//! These constants are the framework's contract baseline from the supplied
//! specification. Validate them after dynamic values and Frame contents have
//! been resolved. Keep semantic limits separate: nested components count
//! toward the message component total, but option/description text does not
//! become Text Display content.
//!
//! Message / Components V2:
//! - 40 total components per message, including nested components.
//! - 4,000 combined characters across Text Displays.
//! - Section: one to three Text Displays and one accessory.
//! - Button Action Row: one to five buttons.
//! - A select menu occupies its own Action Row.
//! - String Select: up to 25 options.
//! - Media Gallery: up to 10 items.
//! - Container: up to 10 direct children.
//! - custom_id: up to 100 characters.
//! - IS_COMPONENTS_V2 flag is required. Traditional message content and
//!   embeds cannot be used in the normal way with a Components V2 message.
//!
//! Modal:
//! - title: up to 45 characters.
//! - up to five top-level components.
//! - input labels: up to 45 characters.
//! - label descriptions: up to 100 characters.
//! - Text Input: up to 4,000 characters.
//! - Select menus: up to 25 options.
//!
//! Dynamic content can invalidate a statically valid Page or Modal, so
//! definition-time validation alone is insufficient.
pub const MESSAGE_MAX_COMPONENTS: usize = 40;
pub const MESSAGE_MAX_TEXT_DISPLAY_CHARS: usize = 4_000;
pub const SECTION_MIN_TEXT_DISPLAYS: usize = 1;
pub const SECTION_MAX_TEXT_DISPLAYS: usize = 3;
pub const BUTTON_ROW_MIN_BUTTONS: usize = 1;
pub const BUTTON_ROW_MAX_BUTTONS: usize = 5;
pub const STRING_SELECT_MAX_OPTIONS: usize = 25;
pub const MEDIA_GALLERY_MAX_ITEMS: usize = 10;
pub const CONTAINER_MAX_DIRECT_CHILDREN: usize = 10;
pub const CUSTOM_ID_MAX_CHARS: usize = 100;

pub const MODAL_TITLE_MAX_CHARS: usize = 45;
pub const MODAL_MAX_TOP_LEVEL_COMPONENTS: usize = 5;
pub const MODAL_LABEL_MAX_CHARS: usize = 45;
pub const MODAL_LABEL_DESCRIPTION_MAX_CHARS: usize = 100;
pub const MODAL_TEXT_INPUT_MAX_CHARS: usize = 4_000;
pub const MODAL_SELECT_MAX_OPTIONS: usize = 25;

// TODO: Confirm whether Serenity exposes any additional version-specific
//       constraints and pin tests to the supported Discord API behavior.
