//! Frame: a dynamic region resolving an ordered collection of components.
//!
//! A dynamic property resolves one value; a Frame resolves heterogeneous
//! Components at render time. Surrounding Page/Container structure remains
//! fixed, and Frame output occupies the exact position where declared.
//!
//! Frame callback receives RenderCtx and Page data and returns Elements.
//! Elements can represent Text, Sections, Containers, Separators, Media
//! Galleries, Files, Buttons, Link Buttons, selects, and button rows where
//! legal. Interactive Frame elements require stable developer-defined keys.
//! A key follows a logical item through sorting, filtering, and pagination;
//! vector position is never identity.
//!
//! Initial support may resolve a complete collection in memory. A paginated
//! data-source abstraction is a later optimization, not a reason to leave
//! pagination state or identity ambiguous.
//!
//! TODO: Define Frame<T, E>, callback, and stable Frame identity.
//! TODO: Support Page-level and permitted Container-level Frames.
//! TODO: Preserve static surrounding content and exact child order.
//! TODO: Collect generated executions recursively and reject duplicate keys.
//! TODO: Define multiple Frame composition explicitly.
pub mod element;
pub mod pagination;
