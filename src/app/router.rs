//! Route Discord interactions to registered executions.
//!
//! Router orchestration: receive an Interaction, extract custom_id, resolve
//! its Store record, locate the Page or Modal definition, reconstruct typed
//! instance data, and invoke the registered callback with a temporary Ctx.
//!
//! Component interactions and Modal submissions are separate routes.
//! Selection values are decoded from the Discord payload and passed as typed
//! Selection<S> alongside original Page data; they are not persisted as part
//! of Page data.
//!
//! Unknown custom IDs, missing definitions/executions, deserialization
//! failures, and unsupported interaction kinds return contextual Result
//! errors. Never treat a valid custom ID as authorization.
//!
//! Frame-generated interactions additionally resolve stable Frame and
//! developer-provided element identity. Never route by a component's current
//! vector position or current pagination page.
//!
//! TODO: Define InteractionRouter<E> and component/Modal dispatch.
//! TODO: Centralize custom-ID lookup and record validation.
//! TODO: Reconstruct PageData<T> separately from selection interaction data.
//! TODO: Resolve Frame identities without relying on ordering or indices.
//! TODO: Define stale-record behavior across deployments/schema changes.
