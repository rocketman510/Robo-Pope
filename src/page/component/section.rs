//! Section: one to three Text Displays beside one accessory.
//!
//! Expose three explicit text slots. Each slot independently accepts static
//! content, a dynamic callback, or absence. Slots must support mixed input
//! types in one call. Do not use one generic array parameter if it forces all
//! positions to share one Into type; use independently generic arguments or
//! another API whose tests prove heterogeneous slots work naturally.
//!
//! The accessory is exactly one Thumbnail or Button. Validate after dynamic
//! resolution: at least one text, no more than three texts, and one
//! accessory. A Button accessory is a normal execution and its ID must be
//! collected recursively.
//!
//! TODO: Define Section<T, E>, Accessory<T, E>, and three-slot setter.
//! TODO: Accept Thumbnail or Button through one ergonomic accessory method.
//! TODO: Resolve slots independently and preserve slot order.
