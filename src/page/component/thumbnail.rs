//! Thumbnail image, normally used as a Section accessory.
//!
//! URL and optional description are static-or-dynamic properties. Spoiler
//! behavior is a static setting unless a later requirement says otherwise.
//! Thumbnail is not a standalone Page/Container component; reject invalid
//! placement explicitly. Validate description length after resolution.
//!
//! TODO: Define Thumbnail<T, E>, resolve URL/description, and enforce legal
//!       Section-accessory-only placement.
