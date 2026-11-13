# Components V2 framework API skeleton

Target: `rocketman510/Robo-Pope`, branch `rewrite/rust`.

This pass revises the earlier documentation-only scaffold against the current
`spec/Page API.md`. It establishes public module relationships, concrete
structs/enums/traits, builder method signatures, callback types, typed registry
handles, Store records, render outputs, Frame identity, pagination state, and
Modal submission types. Functionality is deliberately not implemented;
operations that require behavior remain `todo!()`.

## Error handling

All fallible operations use `anyhow::Result<T>`. There is no custom
framework error enum or custom Result alias. Add `anyhow::Context` at
framework boundaries and preserve the underlying cause and relevant Page,
Modal, execution, component, or Frame identity.

## API organization

- `src/lib.rs`: public library entry point, separate from the Discord bot.
- `src/app/`: App state, host database extension, render/interaction contexts,
  typed registries, routing, Store abstraction, and Serenity event adapter.
- `src/page/`: Page builder and registered typestate, static/dynamic values,
  explicit executions, render outputs, and Discord limits.
- `src/page/component/`: shared Component enum and concrete Components V2
  descriptions, including ordered Container children and fixed Button slots.
- `src/page/select/`: one Page-facing menu enum over string and entity menus.
- `src/page/frame/`: heterogeneous Elements, stable developer keys, and
  semantic pagination budgets.
- `src/page/modal/`: Modal definitions, supported inputs, submission values,
  raw submission callbacks, and typed Serde submission callbacks.

## Design decisions carried forward

- Page data is serialized per render; application extension state is typed
  separately and is never serialized.
- `Page<T, E = ()>` preserves the spec's simple `Page<T>` form while allowing
  callbacks to access an application's typed service bundle when needed.
- Dynamic values use the explicit `dynamic(...)` adapter with the ordinary
  property methods. This avoids overlapping blanket trait implementations.
- Fixed Section slots are independently generic; ButtonRow has five explicit
  optional Button slots.
- Frame-generated interactive elements use developer-defined stable keys,
  never vector positions.
- Container children preserve their declared order, including Frame regions.
- Modal raw callbacks receive `ModalSubmission`; typed callbacks use the
  distinct `.on_submit_typed(...)` method.
- Serenity component builders remain behind the render boundary.
- File display references an attachment; outgoing bytes are handed off
  separately through `RenderedMessage`.
- `MemoryStore` is for tests/development only. Production requires a durable
  Store implementation.

## Known decisions for RED tests / review

- Confirm the ergonomic fixed-slot call sites, especially inferred bare
  `None` and mixed static/dynamic Section slots.
- Confirm the exact Serenity types and method signatures against the pinned
  Serenity `next` branch.
- Decide and test multiple/nested Frame pagination behavior.
- Decide stale Store-record expiry and schema migration policy.
- Verify that the public API examples compile against the crate.
- Do not implement the green phase in this API-skeleton commit.
