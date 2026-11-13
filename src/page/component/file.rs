//! File component and attachment association.
//!
//! File displays an attachment already uploaded with the message. Displaying
//! and uploading are separate operations. RenderedMessage needs an explicit
//! attachment handoff; an attachment:// URL does not upload bytes.
//!
//! Validate that every File component references an attachment included in
//! the outgoing message. Handle duplicate or ambiguous filenames
//! deliberately. Keep the attachment association separate from the
//! component's position in a message.
//!
//! TODO: Define File and outgoing attachment representation.
//! TODO: Make RenderedMessage::send accept or retain upload data.
//! TODO: Validate component-to-attachment association before sending.
