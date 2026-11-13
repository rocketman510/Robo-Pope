//! Modal submission accessors and Serde deserialization.
//!
//! ModalSubmission wraps values returned by Discord for the stable input
//! IDs defined by ModalInput. Provide typed accessors for text, string and
//! entity selections, uploaded attachments, radio selections, checkbox
//! groups, and individual checkbox state.
//!
//! Desired ergonomic contract: deserialize::<Form>() where component IDs
//! map to fields in a user-defined Serde struct. Use Serde rather than a
//! second serialization system. Missing or invalid values produce an error
//! identifying Modal ID and component ID.
//!
//! Entity selections preserve typed IDs and resolved Discord records.
//! File Upload values are attachment IDs/references. Never turn a malformed
//! ID parse into an all-zero fake entity; parse failures are errors.
//!
//! TODO: Define ModalSubmission and typed accessors.
//! TODO: Build a Serde map and deserialize<F>().
//! TODO: Validate missing, wrong-kind, malformed, and invalid values.
