//! Shared framework errors.
//!
//! Fallible framework operations return Result. Errors carry relevant
//! definition, execution, component, and Frame context, while preserving
//! Serenity, serialization, and Store errors as sources.
//!
//! Required categories include duplicate/reserved IDs; unknown Page, Modal,
//! execution, or custom ID; registry data-type mismatch; serialization and
//! deserialization failure; invalid component structure; Discord limit
//! violations; Frame callback/key/pagination failures; missing or wrongly
//! typed Modal submission values; Store failure; and Discord API failure.
//!
//! Framework invariants fail explicitly. Do not panic, unwrap, silently
//! substitute defaults, or fabricate placeholder entity IDs for invalid
//! input.
//!
//! TODO: Define FrameworkError and crate-wide Result<T>.
//! TODO: Preserve source errors and contextual IDs.
//! TODO: Distinguish invalid user configuration from Discord/network errors.
