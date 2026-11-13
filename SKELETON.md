# Components V2 framework API skeleton

Target: `rocketman510/Robo-Pope`, branch `rewrite/rust`.

This pass establishes public module relationships, concrete structs/enums/traits,
builder signatures, callback types, typed registries, Store records, render
outputs, Frame identity, pagination state, and Modal submission types. Functionality
is deliberately not implemented; behavior methods remain `todo!()`.

## Error handling

All fallible operations use `anyhow::Result<T>`. There is no custom framework
error enum or custom Result alias. Add `anyhow::Context` at framework boundaries
and preserve the underlying cause and relevant Page, Modal, execution, component,
or Frame identity.

## API organization

- `src/lib.rs`: public library entry point, separate from the Discord bot.
- `src/app/`: monolithic App ownership, database handle, render/interaction
  contexts, typed registries, routing, Store abstraction, and Serenity event adapter.
- `src/page/`: Page builder and registered typestate, static/dynamic values,
  explicit executions, render outputs, and Discord limits.
- `src/page/component/`: shared Component enum and concrete Components V2
  descriptions, including ordered Container children and fixed Button slots.
- `src/page/select/`: one Page-facing menu enum over string and entity menus.
- `src/page/frame/`: heterogeneous Elements, stable developer keys, and
  semantic pagination budgets.
- `src/page/modal/`: Modal definitions, supported inputs, submission values,
  raw submission callbacks, and typed Serde submission callbacks.
- `src/tests/`: separately filed RED-phase unit tests, grouped by API area.

## Design contracts

- `Page<T>` has one generic parameter: the serialized data shared by that
  Page's components and executions.
- App owns application-wide state. `Ctx` and `RenderCtx` are temporary,
  App-linked contexts; they are not separate service containers or serialized data.
- Page data is serialized per rendered instance and reconstructed before
  its execution callback runs. Application state is not included in Page data.
- Dynamic values use the explicit `dynamic(...)` adapter with ordinary property
  methods.
- Executions are explicit and have stable developer-defined IDs.
- Fixed Section slots and ButtonRow slots represent Discord's structural limits.
- Frame-generated interactive elements use developer-defined stable keys,
  never vector positions.
- Container children preserve their declared order, including Frame regions.
- Modal raw callbacks receive `ModalSubmission`; typed callbacks use the
  distinct `.on_submit_typed(...)` method.
- Serenity component builders remain behind the render boundary.
- File components reference attachments; outgoing bytes are handed off separately.
- `MemoryStore` is for tests/development only. Production requires durable storage.

## RED test policy

Every unit test has a nearby first-principles review comment. The suite focuses on
caller-visible contracts, typed boundaries, limits, persistence, and error cases.
It avoids live Discord dependencies, arbitrary snapshots, and tests that only
repeat constant values. A few unit tests inspect the internal declarative
component sequence because order and fixed slots are themselves contracts; they
should be complemented by rendered-output tests once rendering is implemented.

See `spec/RED Test Plan.md` for the inventory and critique of every test.

## Known API gap

The examples in `spec/Page API.md` access application repositories through App,
such as `ctx.app.db.users`. The current Database type only contains the framework
Store. The host database/repository shape has not been defined, so no invented
`users` API is exposed by this skeleton.

## Known decisions for later review

- Confirm exact Serenity types and builder signatures against the pinned Serenity
  `next` branch.
- Decide multiple/nested Frame pagination behavior.
- Decide stale Store-record expiry and schema migration policy.
- Verify public API examples compile against the crate.
- The RED suite should compile first with `cargo test --no-run`; behavior tests
  should then fail against the unimplemented `todo!()` methods.
- Do not implement the green phase in this commit.
