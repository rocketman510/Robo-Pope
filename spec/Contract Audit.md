# Framework Contract Audit

Branch reviewed: `rewrite/rust`

Primary specification: [Page API.md](Page%20API.md)

This audit records the API contract implied by the specification and compares it with the Rust API on the reviewed branch. It is not an implementation-completion claim. A contract marked **Open** must be resolved before tests assert behavior that depends on it.

## 1. Canonical public model

| Concept | Contract | Current branch |
|---|---|---|
| `App` | Owns the registries, persistence Store, HTTP client, and interaction router. Clones share the same owned state. | Implemented as a non-generic `App`; `Clone` shares database, registries, HTTP, and router. |
| `Page<T>` | Mutable builder; `T` is the Page's shared data type. | Implemented with one generic parameter. |
| `RegisteredPage<T>` | Immutable registered definition. Rendering is only available after registration. | Implemented as a typed handle around `Arc<PageDefinition<T>>`. |
| `Modal<T>` / `RegisteredModal<T>` | Same builder/registered typestate model as Page. | Implemented with one generic parameter. |
| `Frame<T>` | Dynamic, ordered, heterogeneous region with a stable developer-defined ID. | Implemented as a resolver plus ID and pagination flag. |
| `Ctx` | Temporary interaction context. It must not be stored in definitions or persistence. | Shape exists; construction and all response methods are TODOs. |
| `RenderCtx` | Temporary render context carrying App and definition ID. | Implemented. |

### Specification mismatches that must not be hidden

1. Several examples instantiate `Page<T, MyDatabase>` and `Modal<T, MyDatabase>`, but the Rust API defines `Page<T>` and `Modal<T>`.
2. The App example shows `App<E = ()>`, while Rust defines a non-generic `App`.
3. Examples access `ctx.app.db.users` and `ctx.app.db.user_level`. The current `Database` only exposes the framework `Store`; it has no user repositories or host-database generic. Those examples are not compilable against the current branch.
4. Both `Frame::paginate()` and `Page::paginate()` exist. The spec demonstrates Page-level pagination but does not precisely define how that flag interacts with per-Frame pagination.
5. The spec documents `render(app, data)`, while the API additionally exposes `render_pages(app, data)`. The exact relationship between the single-message render and the full pagination result needs a test-backed definition.

The current Rust API is one-generic and App-owned. The repository must make the spec examples agree with that model, or deliberately change the Rust API before treating those examples as contractual.

## 2. Builder and registration

Required behavior:

- Page and Modal IDs are non-empty and unique within their respective registries.
- Registration consumes the mutable builder and returns an immutable typed handle.
- Registered definitions are retained by App registries and remain queryable through cloned App handles.
- Registry storage supports different data types in one registry. A typed query returns an error for unknown IDs and data-type mismatches.
- Page component order and Container child order are preserved.
- Fixed-size slots remain fixed-size: Section has three independent optional text slots; ButtonRow has five optional Button slots. An empty ButtonRow and a Section with no Text Display are invalid.
- Execution IDs are unique within their owning definition. Frame IDs are non-empty and unique within their owning Page. Interactive Frame elements require stable developer keys; identities must not depend on current list index.

Current branch status: Page/Modal builder registration and typed registries are present. Page registration validates individual static components. Duplicate execution IDs and duplicate Frame IDs are not currently checked during registration.

## 3. Values and callbacks

Required behavior:

- A property accepts either its ordinary Rust value or a `Dynamic<V, T>` callback.
- A dynamic callback receives a temporary `RenderCtx` and shared Page data (`Arc<T>`), and runs during rendering.
- Dynamic property callbacks are not interactive executions and do not create Store records.
- The original error from a dynamic callback is preserved.
- Interactive executions are explicit callbacks with a stable developer ID. Page data and interaction payload are separate callback arguments.
- Callback closures remain in registered definitions and are never serialized.

Current branch status: `Value`, `Dynamic`, `IntoValue`, `Slot`, `IntoSlot`, `Execution`, and the typed selection payload model exist. Existing tests cover basic static/dynamic conversion and callback shapes. The render pipeline that actually resolves component properties is still missing.

## 4. Component composition and validation

Required behavior after dynamic values and Frame contents are resolved:

- Produce valid Serenity Components V2 structures while keeping Serenity builders at the rendering boundary.
- Preserve the declared ordering of components and nested Container children.
- Resolve all display properties, including labels, styles, disabled states, descriptions, URLs, options, and accent colors.
- Validate the complete message tree, not only one component at a time.
- Enforce message-wide limits: 40 total components including nested components and a combined 4,000-character Text Display budget.
- Enforce Section, Button Action Row, Container, String Select, and Media Gallery limits.
- Return contextual `anyhow::Result` errors. Do not panic, silently truncate, or manufacture fallback values.

Current branch status: individual component definitions and a component-level validator exist. The validator treats unresolved dynamic text as zero characters and does not itself validate the complete Page-wide component/text budgets. Full post-resolution validation remains part of the missing renderer contract.

## 5. Page rendering and output

Required behavior:

- `RegisteredPage::render` resolves dynamic properties and Frames, validates the final structure, persists every interactive execution reference, and returns a `RenderedMessage`.
- `RegisteredPage::render_pages` returns the complete set of rendered Frame pages and a valid current index.
- Render output contains the component tree, outgoing attachment bytes, and pagination metadata where applicable.
- File components reference attachments by filename; file bytes are attached separately.
- Rendered output must retain all parts needed by the caller. A convenience conversion that consumes a `RenderedMessage` must not silently discard attachments or pagination state.
- A render failure identifies the Page and relevant component/Frame/execution and preserves its original cause.

Current branch status: output structs and accessors exist. Both Page rendering methods are TODOs. `RenderedMessage::into_parts` now preserves the component tree, outgoing attachments, and pagination metadata. The component-only `into_components` convenience method explicitly discards the other two parts.

## 6. Frames and pagination

Required behavior:

- Frame resolver receives the same render context and Page data as a dynamic property.
- A Frame produces ordered heterogeneous elements. Stable element keys are used for interactive identities.
- Pagination is opt-in and splits only between complete elements.
- Every page retains the Page's static components before and after the Frame.
- Static components and framework navigation controls are reserved before allocating Frame elements.
- The planner maximizes complete elements per message while respecting nested component and text budgets.
- An element that cannot fit by itself returns a descriptive error.
- Pagination state identifies the Frame, current index, and page count and is recoverable when navigation is used.
- Navigation cannot move before the first or after the last page.
- Container-contained Frames must preserve Container child order and observe the same limits.

Current branch status: Frame definitions and a standalone pagination planner exist. The planner has unit tests for element boundaries and basic budgets. The Page render path does not yet connect Frame resolution, pagination, static content, navigation, execution persistence, or rendered output. Nested Frame behavior and Page-level versus Frame-level pagination flags are not fully specified.

## 7. Modals and submissions

Required behavior:

- Modal data is serialized when the Modal is rendered/opened and restored when submitted.
- Submitted values are passed separately as `ModalSubmission`.
- Modal input IDs are non-empty, stable, and unique. A Modal requires a submit execution and obeys title, top-level component, label, description, option, and text-input limits.
- Raw submit callbacks receive `Ctx`, original Modal data, and `ModalSubmission` separately.
- Typed callbacks deserialize submission fields by input ID through Serde. Missing or invalid values return errors.
- Opening a Modal is the initial response to an existing interaction, not a message edit or follow-up.

Current branch status: builders, input definitions, basic submission accessors, and String/Boolean Serde conversion exist. Typed deserialization explicitly rejects entity and file values. `RegisteredModal::render` is TODO. The public builder has no dedicated convenience methods for every input variant listed in the spec; those variants can currently be assembled through `ModalInput`.

## 8. Interaction context and routing

Required behavior:

- Gateway interactions are forwarded to App.
- The router distinguishes component interactions from Modal submissions.
- It resolves the custom ID through the Store, validates the record kind, finds the registered definition and execution, restores the original data type, and invokes the callback.
- String, User, Role, Mentionable, and Channel Select payloads remain distinct typed values.
- Modal submission values are passed separately from original Modal data.
- Unknown IDs, wrong definition kinds, unknown executions, malformed payloads, deserialization errors, and Store failures return contextual errors.
- A valid custom ID identifies an execution but is not authorization.
- Ctx exposes interaction user/guild/channel information when present and provides distinct initial reply, defer, component-message edit, original-response edit, follow-up, delete, and Modal-open operations.

Current branch status: `App::dispatch_interaction`, `InteractionRouter`, `Ctx`, and the Serenity Gateway adapter surfaces exist. Router dispatch, Ctx construction/metadata/response methods, and Gateway forwarding are not implemented.

## 9. Persistence and errors

Required behavior:

- Store records contain the custom ID, definition kind and ID, execution ID, serialized original data, optional Frame identity, optional pagination state, and schema version.
- IDs are deterministic for identical inputs and change when any identity input changes. They remain within Discord's 100-character limit.
- Identical Store inserts are idempotent; conflicting records for one custom ID return errors; resolving unknown IDs returns `None`; removing missing IDs is harmless.
- Durable Store implementations preserve records across process restarts. `MemoryStore` is only a test/development implementation.
- All fallible public operations use `anyhow::Result`. Add context without converting errors to strings or discarding the source. Ordinary failures must not panic.

Current branch status: Store trait, MemoryStore, record model, deterministic custom-ID derivation, and tests exist. The current hash uses a custom two-word FNV-style construction rather than the specification's suggested collision-resistant hash (for example BLAKE3); confirm whether that is acceptable before treating it as final.

## 10. Red-phase test matrix

The existing `src/tests` suite covers basic App/registry behavior, builder ordering, common component definitions, execution callback shapes, basic Frame planning, Modal submission accessors, selection payload types, Store behavior, value conversion, and selected component limits.

The following contract tests are still required:

| Area | Required red-phase tests |
|---|---|
| Page render | Static component render; dynamic value is evaluated with App/data; dynamic error is preserved with Page context; final dynamic text and aggregate budgets are validated; execution records are persisted. |
| Rendered output | Attachments and pagination metadata survive ownership conversion; single-message and multi-page render results agree on current index. |
| Frames | Frame resolver is invoked; stable keyed execution identity survives page movement; duplicate/empty Frame IDs fail; static components surround every page; nested Container Frames preserve order; pagination navigation has both bounds. |
| Execution registration | Duplicate execution IDs fail; link buttons never persist executions; ordinary buttons and each Select kind persist the correct kind and ID. |
| Modal render | Modal data is persisted; generated custom ID resolves to a Modal record; dynamic labels/options resolve; all supported input structures render; Modal constraints are enforced after resolution. |
| Typed Modal values | Serde conversion tests cover Text, strings, bool, entity selections, and file references, or the contract explicitly narrows supported typed fields. |
| Router | Component and Modal-submit routes; missing record; wrong definition kind; unknown definition/execution; data deserialization failure; each selection payload kind; original Modal data and submitted values remain distinct. |
| Ctx | Construction copies App/HTTP and selection; user/guild/channel accessors cover guild and DM interactions; each response operation chooses the correct initial/edit/follow-up/modal endpoint. |
| Gateway | InteractionCreate is forwarded to `App::dispatch_interaction`; routing errors are surfaced rather than logged as a successful dispatch. |
| Validation | Whole-message nested component counts and aggregate text; dynamic properties; Frames inside Containers; every published Discord limit and boundary. |

### Tests added in the current red-phase batch

The following tests have now been committed and registered in `src/tests/mod.rs`:

- `page_render_resolves_dynamic_values_with_page_data_and_app_context`
- `page_render_preserves_dynamic_callback_errors_and_adds_page_context`
- `page_render_validates_the_combined_text_display_budget_after_resolution`
- `page_render_persists_button_execution_identity_and_page_data`
- `render_pages_keeps_static_content_and_splits_frame_elements_in_order`
- `modal_render_persists_modal_data_and_returns_a_modal_response`
- `page_registration_rejects_duplicate_execution_ids`
- `page_registration_rejects_duplicate_frame_ids`
- `rendered_message_parts_conversion_preserves_attachments_and_pagination`
- `context_construction_retains_app_http_and_absent_ping_metadata`
- `context_retains_the_typed_selection_payload`
- `router_returns_an_error_for_an_unsupported_interaction_kind`

These tests are intentionally ahead of the unfinished renderer, Modal renderer, context, router, and registration validation paths. The output-parts conversion test exercises the completed `RenderedMessage::into_parts` API. The rest are expected to expose missing behavior once confirmed by an actual test run.

Do not begin the green phase merely because these tests compile. The red phase is complete only after the test command has been run against this branch and failures are confirmed to be behavioral contract failures rather than compile failures.
