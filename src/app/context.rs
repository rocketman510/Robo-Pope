//! Render-time and interaction-time contexts.
//!
//! RenderCtx is temporary and supplied while dynamic Page properties and
//! Frames resolve. It exposes App<E>, definition identity, render-instance
//! data, and an internal collector for discovered execution references.
//!
//! Ctx is temporary and belongs to one Discord interaction. It supplies the
//! application and typed interaction information, plus common operations:
//! reply, defer, acknowledge, edit the component message, edit the original
//! response, follow up, delete the response, and open a Modal.
//!
//! Original Page/Modal data and interaction submission data are separate.
//! Select values come from the interaction payload; ModalSubmission contains
//! submitted form values. Neither should be silently merged into T.
//!
//! Accessors expose user, guild/channel, interaction ID, message, and
//! selection values. Operations invalid for an interaction kind return
//! descriptive errors or Option rather than panicking.
//!
//! A valid custom ID identifies an execution, not its authorization. The
//! callback remains responsible for permission checks.
//!
//! Intended type shape (illustrative):
//! \`\`\`rust,ignore
//! struct RenderCtx<'a, T, E> {
//!     app: &'a App<E>,
//!     definition_id: &'a str,
//!     instance_data: &'a T,
//!     execution_collector: /* render-local collector */,
//! }
//! struct Ctx<'a, E> {
//!     app: &'a App<E>,
//!     interaction: &'a Interaction,
//!     selection: /* optional typed interaction payload */,
//! }
//! \`\`\`
//!
//! TODO: Define RenderCtx<'a, T, E> and Ctx<'a, E>.
//! TODO: Prefer borrowing Page data in async dynamic callbacks instead of
//!       requiring Clone on every T, if lifetimes permit.
//! TODO: Model response acknowledgement state to prevent multiple initial
//!       responses.
//! TODO: Preserve Serenity failures as framework errors.
