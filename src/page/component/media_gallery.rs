//! Media Gallery and individual gallery items.
//!
//! A gallery contains image/video URLs and optional descriptions, each
//! static-or-dynamic. Items may have spoiler behavior. Require at least one
//! item and enforce Discord's ten-item maximum after resolving content.
//! Descriptions have their own limits and do not consume the Text Display
//! character budget. Each resolved item remains a complete pagination unit.
//!
//! TODO: Define MediaGallery<T, E> and MediaItem<T, E>.
//! TODO: Support add_media/add_item composition and dynamic properties.
//! TODO: Validate nonempty gallery, item count, descriptions, and URLs.
