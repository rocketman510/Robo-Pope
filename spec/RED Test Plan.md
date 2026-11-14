# RED Phase Test Plan

Basis: `spec/Page API.md`. The test suite follows the public Page API and treats it as the authoritative contract. Tests must not introduce a second, contradictory design.

## Test-first rules

Tests are focused, independent, repeatable, self-validating, and deterministic. Discord HTTP behavior is tested through a local TCP server, never a live Discord connection.

The suite must compile before runtime RED failures are evaluated. A compilation error, malformed fixture, panic, timeout, or unrelated Serenity builder failure is a test defect, not an acceptable RED result.

Do not add test-only API requirements that are absent from `Page API.md`. When a public contract changes, update the matching tests and this plan in the same change.

## 1. Page, App data, and registration

Relevant tests:

- `src/tests/app.rs`
- `src/tests/database_contract.rs`
- `src/tests/support.rs`
- `src/tests/builder.rs`

Contracts:

- `App<D>` exposes the application's concrete database value directly as `app.db`.
- The application defines `D`; named repositories and fields are accessed directly, including from dynamic property and execution callbacks.
- The framework must not require `Database::get::<T>()`, a type-erased host database wrapper, or string-based table lookup.
- The application database and the framework interaction Store are separate dependencies.
- Cloning App preserves its concrete database handle and registered state.
- One App can hold registered Pages with different generic Page data types.
- Page and Modal definitions share one globally unique ID namespace.
- Registration consumes builders and returns immutable registered definitions.
- The public registry path `app.pages.query::<T>(id)?.render(...)` and corresponding Modal query/render path are tested end to end.
- Unknown definition IDs and duplicate definition IDs return errors.

## 2. Static and dynamic properties

Relevant tests:

- `src/tests/value.rs`
- `src/tests/property_contract.rs`
- `src/tests/builder.rs`
- `src/tests/rendering.rs`

Contracts:

- A property accepts its ordinary static value or an asynchronous callback returning that value.
- A callback can be passed directly. `dynamic(...)` is an optional helper, not a required wrapper.
- Dynamic properties receive the render context and the Page's shared data.
- Dynamic property callbacks return `Result`; their errors are preserved and annotated with definition context.
- Dynamic property callbacks cannot use interaction-response operations because no interaction is active.
- Fixed Section text slots accept direct strings, direct callbacks, or `None`.
- Button label, style, optional emoji, and disabled state all accept static or dynamic values.
- LinkButton label, URL, optional emoji, and disabled state all accept static or dynamic values.
- Container accent color, Separator spacing and divider, Thumbnail URL/description/spoiler, and Media Gallery URL/description/spoiler accept dynamic values.
- String Select placeholder, option label, description, and optional emoji accept dynamic values.
- Static and dynamic properties use the same public method or constructor argument.

## 3. Components V2 and fixed composition

Relevant tests:

- `src/tests/components.rs`
- `src/tests/validation.rs`
- `src/tests/rendering.rs`

Contracts:

- Page and Container composition preserves the order in which children are added.
- Section exposes three fixed Text Display slots, accepts direct values and `None`, and requires at least one populated slot.
- A Section accessory is a Thumbnail or a fully configured Button.
- A Button constructor has six arguments, in order: label, static execution ID, style, optional emoji, disabled Boolean, and execution callback.
- A LinkButton has label, URL, optional emoji, and disabled state. It has no execution callback or routing record.
- A rendered LinkButton has a URL and disabled property but no `custom_id`; rendering it does not add a routing record. A sibling callback Button still gets its record.
- A Button row has five fixed slots. Each slot accepts Button, LinkButton, or `None`; present Buttons are passed directly without `Some(...)`.
- Media Gallery exposes both `set_media(...)` and `add_media(...)`, preserves item order, and validates the one-to-ten item range.
- File construction accepts a complete `attachment://` URI, stored as `attachment_uri`. File bytes are attached separately.
- Container supports the component composition methods allowed by Discord.

## 4. Selection menus

Relevant tests:

- `src/tests/selection.rs`
- `src/tests/selection_contract.rs`

Contracts:

- A Page uses one selection-menu entry point for String, User, Role, Mentionable, and Channel Selects.
- String Select preserves placeholder, option label, description, optional emoji, submitted value, min selection count, and max selection count.
- User, Role, Mentionable, and Channel Selects each render their distinct Discord component type and preserve placeholders and selection bounds.
- Selection callbacks receive selected values or typed entities as appropriate.
- Selection values supplied by Discord are interaction data, not serialized Page data.
- The complete message-side selection menu may be disabled.
- String Select options do not have an individual disabled field. Static or dynamic attempts to configure one are rejected rather than silently discarded.
- Static and dynamic option descriptions exceeding Discord's limit are rejected.
- Modal selection inputs do not expose disabled state.

## 5. Frames

Relevant tests:

- `src/tests/frame.rs`
- `src/tests/frame_contract.rs`
- `src/tests/rendering.rs`

Contracts:

- A Frame is supplied as a dynamic callback on the Page and returns a collection of heterogeneous framework elements.
- The Frame uses the containing Page's generic data and rendering context.
- Frame elements are expanded at the Frame's declared position and preserve the returned order.
- Multiple Frames may appear in one Page.
- A Frame callback is evaluated once for a Page render.
- Frame elements use the same component property and execution contracts as components declared directly on the Page.
- Interactive Frame elements use their developer-defined execution IDs. Frame index, array position, named Frame IDs, and extra element keys are not part of custom-ID derivation.
- Duplicate execution IDs within a Page are rejected, including duplicates emitted by a Frame.
- The completed Page is validated against Discord's message limits after Frame expansion.
- Frames do not automatically paginate. There is no `Frame::paginate()`, `Page::paginate()`, `render_pages()`, page index, offset, range, cursor, Previous/Next generation, pagination state, or pagination compare-and-set contract in this API.

## 6. Modals

Relevant tests:

- `src/tests/modal.rs`
- `src/tests/modal_contract.rs`

Contracts:

- Modal builders register immutable definitions in the App's shared Page/Modal ID namespace.
- Each Modal input is serialized as a Discord Label component wrapping the actual input. Tests verify wrapper and nested component types for Text Input, String Select, entity selects, Checkbox, Checkbox Group, Radio Group, and File Upload.
- Modal input Labels and their Text Input placeholders use their documented length limits. Static violations are rejected during registration; dynamic violations are rejected after resolution.
- Modal rendering validates and builds the complete payload before persisting its submit record. Failed rendering leaves no orphaned record.
- Modal input components do not send a disabled property.
- Typed Modal submissions deserialize through Serde using developer-defined input IDs and struct field names.
- Supported typed values include text, String Select values, entity IDs, Mentionable tagged IDs, Radio Group values, Checkbox Group values, Boolean Checkbox values, and uploaded-file metadata.
- Raw ModalSubmission access remains available when application code needs direct access to the submission.
- Missing, malformed, or incompatible required submission values return errors.

## 7. Interaction context and execution routing

Relevant tests:

- `src/tests/execution.rs`
- `src/tests/execution_wiring.rs`
- `src/tests/interaction.rs`
- `src/tests/ctx_http.rs`
- `src/tests/routing_contract.rs`

Contracts:

- Every interactive component has an explicit asynchronous callback and a developer-defined execution ID unique within its Page or Modal.
- Registration indexes callbacks; the gateway handler forwards incoming interactions to App dispatch.
- Component and Modal dispatch restore serialized data into the registered definition's own generic type.
- Unknown custom IDs, malformed stored data, and interactions routed to the wrong definition kind return errors.
- A custom-ID hash collision or conflicting insertion is rejected by the Store.
- The framework Context exposes the App and the relevant Serenity interaction information.
- `reply`, `defer`, `edit_message`, `edit_original_response`, `followup`, `delete_response`, and `open_modal` preserve their distinct Discord semantics.
- Initial interaction responses are mutually exclusive. A second initial response returns an error.
- HTTP response behavior is tested through a local test server.

## 8. Persistent interaction Store

Relevant tests:

- `src/tests/store.rs`
- `src/tests/database_contract.rs`
- `src/tests/rendering.rs`

Contracts:

- The framework Store is SQLite-backed and persists routing records across process restarts.
- A routing record has exactly four fields: deterministic hash, definition ID, execution ID, and serialized Page or Modal data.
- The custom ID is deterministically computed from exactly the definition ID, execution ID, and serialized data.
- Identical inputs yield identical hashes across repeated renders and process restarts.
- The generated custom ID fits Discord's limit at both derivation and rendered-component boundaries.
- Inserting an identical record is idempotent.
- A hash collision or conflicting record is an error; a record is never overwritten with different callback identity or data.
- The Store is a routing lookup, not a user-session or message-session database.
- Routing records do not contain user ID, message ID, Frame identity, pagination state, timestamp, or expiration fields.
- The framework does not automatically expire records or use TTL-based session cleanup.
- LinkButtons do not create routing records.

## 9. Errors and Discord validation

Relevant tests:

- `src/tests/validation.rs`
- `src/tests/rendering.rs`
- `src/tests/selection_contract.rs`
- `src/tests/modal_contract.rs`

Contracts:

- Invalid structures and unsupported component properties return errors rather than panic or silently changing the requested UI.
- Static violations that can be known at registration are rejected there. Dynamic violations are rejected after the value resolves and before sending.
- The message-wide component count accepts forty components and rejects forty-one.
- Message component limits, the 4,000-character Text Display limit, the Container ten-direct-child limit, Section slot requirements, Action Row slot requirements, Media Gallery item counts, selection menu option counts, and Modal input limits are enforced.
- The final resolved Page is validated after dynamic properties and Frame contents are expanded.
- A component tree that exceeds Discord's message limits returns an error. The framework does not silently truncate it or implicitly paginate it.

## Test ordering

The module declarations in `src/tests/mod.rs` follow the API's progression: Page and registration, property conversion, component composition, selection menus, Frames and rendering, Modals, interaction Context and execution routing, App database and Store persistence, then validation.

## RED interpretation

A failing runtime assertion is an expected RED result only if the test compiled and reached the contract assertion. Do not replace assertions with unconditional failures, invert tests to bless current stubs, or weaken assertions merely to get a green build.

No GREEN implementation is claimed by this plan. A later GREEN change must satisfy the tests without weakening their assertions or introducing public APIs that contradict `spec/Page API.md`.
