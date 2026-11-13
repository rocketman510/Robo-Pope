//! Discord Components V2 and Modal constraints from spec/Page API.md.
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

// Count nested components recursively, but keep semantic budgets separate:
// descriptions and select labels do not consume the Text Display character
// budget. Validate after resolving dynamic properties and Frame contents.
