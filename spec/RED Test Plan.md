# RED Phase Test Plan

Basis: `spec/Page API.md`. This is the executable contract checklist for `rewrite/rust-red-tests`. It separates contract decisions from the behavior that a later GREEN phase must implement.

## Test-first rules

Every new test is FIRST: focused, independent, repeatable, self-validating, and timely. Each test has a nearby first-principles comment. Fixtures are deterministic. Discord HTTP behavior is tested with a local TCP server, never a live bot or network service.

The suite must compile before runtime RED failures are evaluated. A compilation error, invalid fixture, panic, timeout, or unrelated Serenity builder failure is a test defect, not an acceptable RED result.

## Contract decisions recorded in Page API.md

- Every Page/Modal execution callback is indexed during registration, and the gateway handler forwards InteractionCreate to App dispatch.
- Modal rendering validates and builds the complete payload before persisting its submit record; failed rendering leaves no orphan.
- Host application database state is separate from the framework interaction Store. `Database::get::<T>()` retrieves the host application's concrete database aggregate.
- Production Store records must survive process restart. `MemoryStore` is process-local and development/test only.
- Direct asynchronous closures are valid inputs to dynamic component properties and fixed Section slots. `dynamic(...)` is optional.
- Page and Modal IDs share one global namespace.
- Every interactive descendant inside a Frame element requires a stable developer key, including Section accessories and Container descendants.
- A Page supports at most one paginated Frame in its component tree. It may be nested inside a Container; static siblings remain on every generated page.
- Framework pagination uses reserved Previous/Next executions, persists its current state, uses atomic compare-and-set, and rejects stale or out-of-range navigation.
- Dynamic property values are resolved and measured before page boundaries are chosen.
- Modal Serde mapping is defined for text, string selections, booleans, entity IDs, mentionables, and uploaded-file metadata.
- Unsupported per-option disabled String Select state is rejected. Disabling the complete menu remains supported.
- Modal input limits are checked before registration when static and after resolution when dynamic.
- Ctx response methods have distinct Discord semantics and are tested through a local HTTP double.

## Existing tests retained

The existing focused modules continue to cover App and definition registries, builders, component composition, basic value resolution, selection payload typing, Modal accessors, Store idempotency/conflicts, validation limits, and simple Frame planning. New tests extend those contracts end-to-end.

## New behavioral suites

### `src/tests/database_contract.rs`
- Host database concrete type retrieval succeeds and a wrong type errors.
- Host database and framework Store are separate App dependencies.
- App clones preserve the same host database handle.
- Production Store durability is required; MemoryStore does not satisfy it.

### `src/tests/execution_wiring.rs`
- Page registration indexes every developer callback.
- Component dispatch restores Store data and invokes the typed callback.
- Modal registration indexes its submit callback and Modal dispatch invokes it.
- Gateway handler forwards InteractionCreate to App dispatch.
- Page and Modal IDs are globally unique in either registration order, including concurrent registration.

### `src/tests/ctx_http.rs`
- Reply, defer, component-message update, original-response edit, follow-up, response deletion, and Modal opening use distinct Discord routes and response types.
- A second initial response is rejected.
- A local TCP HTTP double captures requests and returns deterministic responses.

### `src/tests/modal_contract.rs`
- String Select, entity Select, Checkbox, Checkbox Group, Radio Group, and File Upload Modal inputs render.
- Failed dynamic rendering leaves no orphaned Store record.
- Entity IDs, Mentionable discriminants, and uploaded-file metadata survive typed Serde conversion.
- Text input length ranges, input counts, duplicate IDs, and all option-count limits are enforced.
- Modal option descriptions are validated against Discord's limit.

### `src/tests/pagination_contract.rs`
- Previous/Next controls are generated and bounded.
- Navigation records persist Page/Frame identity and PaginationState.
- Navigation advances state atomically and rejects stale/out-of-range clicks.
- Nested Container Frames paginate in place and preserve static siblings.
- A Page rejects multiple paginated Frames.
- Resolved dynamic property cost is included in page planning.
- Interactive descendants of Sections and Containers require stable keys.
- Stable keys preserve custom IDs when Frame element order changes.
- Frame resolution is performed once per render plan.

### `src/tests/selection_contract.rs`
- Unsupported disabled String Select options are rejected after static or dynamic resolution.
- Whole-menu disabled state remains supported.

### `src/tests/store.rs`
- Pagination-state compare-and-set updates only the expected current state and rejects stale updates.

### `src/tests/value.rs`
- Direct asynchronous closures work for Text Display values, Button properties, and Section slots without `dynamic(...)`.

## RED interpretation

A failing runtime assertion is an expected RED result only when the test compiled and reached the contract assertion. Do not replace contract assertions with unconditional failures, invert tests to bless current stubs, or weaken assertions to get a green build.

No GREEN implementation is claimed by this plan. A later GREEN change must make each failing contract test pass without weakening its assertion.
