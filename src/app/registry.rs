//! Typed registries for immutable registered Page and Modal definitions.
//!
//! One App may own Page<UserData>, Page<SettingsData>, and
//! Modal<TicketData>. Public query methods recover RegisteredPage<T> or
//! RegisteredModal<T>; callers never handle erased definitions.
//!
//! Each entry has a definition ID, a private erased interface for routing,
//! and a typed handle for public queries. Erasure is an implementation
//! detail. Registries contain callbacks, not serialized closures. Store
//! records point to definition and execution IDs; the registered definition
//! resolves those IDs to callbacks.
//!
//! Registration rejects duplicate IDs without replacing an existing
//! definition. Page and Modal IDs participate in deterministic custom-ID
//! identity, so their namespace policy must be explicit and collision-free.
//!
//! Intended shape (illustrative):
//! \`\`\`rust,ignore
//! struct PageRegistry<E> { entries: /* HashMap<PageId, ErasedPage<E>> */ }
//! struct ModalRegistry<E> { entries: /* HashMap<ModalId, ErasedModal<E>> */ }
//! trait ErasedPage<E> { /* render / invoke without exposing T */ }
//! trait ErasedModal<E> { /* render / submit without exposing T */ }
//! \`\`\`
//!
//! TODO: Define PageRegistry<E>, ModalRegistry<E>, and erased entry traits.
//! TODO: Preserve concrete T on lookup and return DataMismatch for wrong T.
//! TODO: Make duplicate registration atomic and non-destructive.
//! TODO: Decide whether Page and Modal IDs share one global namespace.
//! TODO: Define safe concurrent lookup and startup registration behavior.
