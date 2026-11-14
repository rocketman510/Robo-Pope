//! Implementation sequence for the Page API contract.
//!
//! 1. Typed App, Page, Modal, and callback definitions.
//! 2. Static/dynamic Value resolution and fixed-slot conversion ergonomics.
//! 3. Components V2 composition, validation, and Serenity rendering.
//! 4. Button execution IDs, four-field Store records, and deterministic routing.
//! 5. String/entity selection menus and typed interaction payloads.
//! 6. Modal registration, inputs, submission routing, and typed Serde forms.
//! 7. Heterogeneous Frame resolution and ordered single-message expansion.
//! 8. Validate the fully resolved message against Discord limits before persisting routes.
//!
//! A Frame does not paginate. It resolves once at its declared position. The
//! framework returns one message or an error when the resolved component tree
//! exceeds Discord limits. Routing identity contains only definition ID,
//! execution ID, and serialized data; callback records contain those three
//! values plus the deterministic hash.
