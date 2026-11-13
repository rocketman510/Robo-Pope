//! Modal blueprint, inputs, registration, and typed submission.
//!
//! Modal is a separate form blueprint, not a message component. It is opened
//! as the response to an existing interaction and produces a separate
//! Modal submission interaction. It has a globally unique ID and its own
//! serialized data type T.
//!
//! Registration consumes Modal<T> and returns RegisteredModal<T>. Rendering
//! serializes associated data and persists the execution reference before
//! opening the Modal. Registered definition owns on_submit; closures are
//! never serialized.
//!
//! Submitted values are interaction data, separate from persisted Modal
//! data. The callback receives Ctx, T, and ModalSubmission separately.
//!
//! Support Text Input, String/User/Role/Mentionable/Channel Select, File
//! Upload, Radio Group, Checkbox Group, and Checkbox. Required Discord Label
//! structures are created internally.
//!
//! Typed deserialization maps stable component IDs to fields in a Serde
//! struct. Missing/invalid values return contextual errors.
//!
//! Intended type shape (illustrative):
//! \`\`\`rust,ignore
//! pub struct Modal<T, E = ()> {
//!     id: ModalId,
//!     title: String,
//!     inputs: Vec<ModalInput>,
//!     on_submit: Submit<T, E>,
//! }
//! pub struct RegisteredModal<T, E = ()> {
//!     definition: Arc<ModalDefinition<T, E>>,
//! }
//! \`\`\`
//!
//! TODO: Define Modal<T, E>, RegisteredModal<T, E>, and registration.
//! TODO: Keep Modal distinct from Page message components.
//! TODO: Persist Modal data and route submission to registered callback.
pub mod input;
pub mod submission;
