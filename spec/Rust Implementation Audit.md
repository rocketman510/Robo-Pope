# Rust Implementation Audit

## Scope

- **Repository:** rocketman510/Robo-Pope
- **Branch audited:** rewrite/rust
- **Branch HEAD at audit start:** aa5ebf1cc241efc1f540101d2bd19b05a62fc57a
- **Contract source:** spec/Page API.md
- **Contract blob at audit start:** e923ad1de5b954445544a58236ae0e6f3156b38c
- **Files reviewed:** all 51 Rust files present on the branch, including the binary, library modules, and in-tree tests.

This is a static implementation audit against Page API.md. No Rust source, test, Cargo manifest, workflow, or specification file was changed. No tests were written. The repository workflow is configured to run cargo test --all-targets, but this audit did not execute that command or independently verify a CI run.

Finding labels:

- **[TEST]** A unit, compile-time, or integration test can demonstrate the failure.
- **[REVIEW]** A test can check examples, but cannot establish the underlying property by itself. Code or algorithm review is also required.
- **[GAP]** The API contract does not define enough behavior to decide what the implementation should guarantee. This needs a specification decision.

## Executive summary

The branch has a substantial amount of real builder and rendering code. Page registration, typed registries, static and dynamic value storage, several Components V2 renderers, in-memory Store operations, and a basic Frame planner exist.

However, the core end-to-end interaction system is not implemented. Both interaction router paths are TODOs, every interaction response method on Ctx is a TODO, and AppHandler does not forward incoming interactions to App::dispatch_interaction. The database abstraction is also only a wrapper around the framework Store, not the application database/repository surface shown in Page API.md. The only Store implementation in the repository is MemoryStore, which is explicitly non-durable.

There are also substantial gaps in Modal rendering, automatic Frame pagination, Frame identity enforcement, component-budget accounting, and the supported dynamic-value conversion API.

The existing tests mostly establish that values and builders can be constructed, registries can store definitions, MemoryStore can store records in the current process, and simple Text Display Frames can be split by the standalone planner. They do not exercise a successful button interaction, a successful select interaction, a Modal submission route, any Ctx response operation, durable persistence, pagination navigation, or the Discord send path. Therefore, a passing cargo test --all-targets result does not establish that the exposed framework works end to end.

---

# A. Test-provable contract failures

These are observable failures for which a focused unit, compile-time, or integration test can be written. No such tests were added during this audit.

## T-01. Neither interaction route is implemented

**Files:** src/app/router.rs, src/app/mod.rs

InteractionRouter::dispatch_component and InteractionRouter::dispatch_modal_submit both contain todo!(). App::dispatch_interaction delegates to these methods.

A valid Component interaction therefore cannot reach its registered Page execution. A Modal submission cannot reach its registered Modal callback. The main behavior described in Page API.md section 9, including resolving the custom ID, finding the definition and execution, deserializing the original data, and invoking the callback, is absent.

The existing interaction test only sends a Ping interaction and verifies that the router returns an unsupported-kind error. It never invokes either TODO route.

**Contract not fulfilled:** Sections 4, 6, 7, and 9. Button, selection, and Modal executions cannot be dispatched.

**Test that would expose it:** Construct a valid Component interaction with a persisted StoreRecord and registered Page, then assert that the expected callback runs with the original data and selected values. Do the equivalent for a Modal submission. Also verify unknown IDs and malformed records return errors rather than panic.

## T-02. All interaction response methods on Ctx are unimplemented

**File:** src/app/context.rs

The following public methods contain todo!():

- reply
- defer
- edit_message
- edit_original_response
- followup
- delete_response
- open_modal

These are the operations explicitly required by Page API.md section 7. Calling one currently panics instead of returning a Result.

The current tests construct Ctx and inspect its metadata. They do not call any response method.

**Contract not fulfilled:** Sections 6 and 7. Applications cannot reply to interactions, defer them, edit messages, send follow-ups, delete responses, or open Modals through the advertised context.

**Test that would expose it:** Exercise every method against an HTTP test double or an integration test and verify the correct Discord response type and request. At minimum, tests should prove that no public operation panics.

## T-03. The gateway handler never dispatches incoming interactions

**Files:** src/app/handler.rs, src/main.rs

AppHandler handles InteractionCreate by printing that routing is not implemented. It does not call App::dispatch_interaction.

The executable in src/main.rs does not use AppHandler. Its private DemoHandler handles Ready and Message events, but has no InteractionCreate handling. It can send a message containing the demo components, but it does not route clicks or Modal submissions back into the framework.

Even after T-01 is implemented, the supplied handler and executable will still not connect Discord interaction events to the router.

**Contract not fulfilled:** Sections 8 and 9. The App is supposed to own the interaction router and receive the incoming interaction.

**Test that would expose it:** Dispatch a simulated InteractionCreate event through AppHandler and assert that the registered execution is invoked. Add a binary-level or handler-level test proving the demo button can be clicked and its callback runs.

## T-04. The App database does not provide the application data API shown in the contract

**Files:** src/app/database.rs, src/app/mod.rs, src/app/context.rs

Database contains only one field: store: Arc<dyn Store>. It is framework persistence for custom-ID execution records. It does not expose application repositories or any general application database interface.

Page API.md uses expressions such as ctx.app.db.users.query(...). That API cannot be written against this Database type. The source comment in database.rs itself identifies repository access as an API gap.

The framework Store and the host application's data database are different responsibilities, but the current public App API uses the name db for the Store-only wrapper.

**Contract not fulfilled:** The App overview and sections 1, 7, and 8. The supplied Page examples cannot access the application data they demonstrate.

**Test that would expose it:** Add a compile-time API test using the documented ctx.app.db.users example. It cannot compile with the current Database type.

## T-05. The repository has no durable Store implementation

**Files:** src/app/store.rs, src/app/database.rs, src/main.rs

Store is an injectable trait, which is a useful extension point. However, the only implementation in this repository is MemoryStore. Its own documentation says it is not durable across restarts. The executable explicitly constructs MemoryStore.

Consequently, the provided executable loses all custom-ID execution records when the process exits. A custom Store supplied by an external application could be durable, but this repository does not provide one or demonstrate one.

**Contract not fulfilled by the supplied runtime:** The design principle that persistence survives restarts, and section 9's persistent custom-ID mapping.

**Test that would expose it:** Persist a rendered button record, stop the process, create a fresh App using the intended production Store, and verify the interaction record is still resolvable. A MemoryStore-only round trip is not a restart-persistence test.

## T-06. Modal rendering implements only Text Input

**File:** src/page/modal/mod.rs

ModalInput exposes Text, StringSelect, UserSelect, RoleSelect, MentionableSelect, ChannelSelect, FileUpload, RadioGroup, CheckboxGroup, and Checkbox.

RegisteredModal::render only handles ModalInput::Text. Every other input variant reaches the wildcard branch and returns an error saying that rendering the input type is not implemented.

The API contract explicitly lists these Modal component types. The existing Modal tests construct several definitions and test accessors, but the only actual Modal rendering test uses a Text Input.

There is also a side effect ordering problem: render persists the Modal submit StoreRecord before it attempts to render the inputs. Rendering an unsupported input therefore returns an error after leaving a persisted record behind.

**Contract not fulfilled:** Section 6, Modal components. The exposed input types cannot be rendered.

**Test that would expose it:** Render one Modal containing each supported input kind and inspect the resulting Discord Label and input components. Include a failed-render test verifying that a failed Modal render does not leave an unusable StoreRecord.

## T-07. Typed Modal deserialization rejects several supported input value types

**File:** src/page/modal/submission.rs

ModalSubmission::deserialize supports Text, Strings, and Boolean. It explicitly errors for Users, Roles, Mentionables, Channels, and Files.

Page API.md describes typed Modal submission as the long-term goal and says values should be converted into appropriate Rust types. The current API exposes on_submit_typed now, but its supported type set is substantially narrower than the ModalSubmission enum and the advertised Modal input set.

This should be tracked as partial implementation rather than treated as complete typed Modal support.

**Contract partially fulfilled:** Section 6, Typed submission data.

**Test that would expose it:** Deserialize forms containing a selected user, selected role, channel, mentionable, and file upload into supported Serde field types. Verify missing and invalid values still return errors.

## T-08. Frame pagination produces page objects but does not provide navigation

**Files:** src/page/builder.rs, src/page/frame/pagination.rs, src/page/render.rs, src/app/store.rs

The planner can split a Frame into a Vec of pages. render_pages returns those pages and attaches a PaginationState to each RenderedMessage. That is not the same as framework-managed navigation.

No previous/next navigation buttons are generated. No internal navigation executions are registered. No route changes the current page. StoreRecord has a pagination_state field, but the render persistence paths always set it to None. No code writes or restores the current page index.

RegisteredPage::render returns only the first entry from render_pages. A caller using render instead of render_pages receives no way to access the remaining pages.

The test named pagination_state_round_trips_through_persistence_format serializes and deserializes a PaginationState value in memory. It does not store that state in Store, recover it, or prove navigation works.

**Contract not fulfilled:** Frame pagination controls and pagination state in section 5, including preventing navigation beyond the first and last page.

**Test that would expose it:** Render a multi-page Frame and assert that framework navigation controls are present. Simulate next and previous interactions, verify the displayed page changes, verify boundaries, and verify the current index survives Store/process reconstruction.

## T-09. Pagination is not applied to Frames nested in Containers

**Files:** src/page/builder.rs, src/page/component/container.rs, src/page/frame/mod.rs

Container exposes add_frame, and the final composition example in Page API.md places a Frame inside a Container and calls paginate on that Frame.

The Container rendering path resolves nested Frame elements and appends them directly to the Container. It never calls the pagination planner and does not inspect the nested Frame's paginate flag. If the expanded children exceed the Container's direct-child limit, rendering errors instead of splitting the Frame across pages.

Top-level Page rendering also rejects more than one top-level Frame. Page API.md does not specify this one-top-level-Frame restriction.

**Contract not fulfilled:** The documented Container/Frame composition and automatic Frame pagination behavior.

**Test that would expose it:** Put more than ten generated children in a paginated Frame inside a Container. Verify that rendering produces valid pages rather than an over-limit Container error.

## T-10. Pagination budget measurement does not match actual rendered component usage

**Files:** src/page/frame/pagination.rs, src/page/builder.rs, src/page/render.rs

The pagination planner calculates budgets before the Frame elements are rendered. Several costs are currently wrong or omitted:

- Dynamic Text Display values are counted as zero characters by pagination::measure. Their actual content is only resolved later.
- Section measurement does not add its nested Text Displays to total_components.
- Button, LinkButton, and Select measurement counts the component but not the Action Row wrapper that the renderer creates.
- The renderer's Section budget counts the Section and its Text Displays but omits its Button or Thumbnail accessory.
- The renderer's Media Gallery budget counts the gallery but not its nested media items, while the planner does count gallery items.

This means the planner and the renderer disagree about component usage. It can place content on a page that later fails validation, and the renderer can undercount nested components when checking the 40-component message limit.

Dynamic content creates another failure mode: a Frame can be planned using zero character cost, then fail after dynamic strings are resolved. The implementation returns an error instead of re-planning the page boundaries using the resolved content.

The current pagination tests primarily use simple Text Display elements with static strings. They do not cover the mixed component types or dynamic values that reveal these differences.

**Contract not fulfilled:** Sections 5 and 12. Pagination must account for nested components and resolved dynamic content, and rendered messages must remain within the 40-component limit.

**Test that would expose it:** Add planner and end-to-end render tests for Sections with three text slots and accessories, Buttons, Link Buttons, Select menus, Media Galleries, nested Containers, and dynamic Text Displays. Assert both that page splitting maximizes valid content and that the final rendered component tree never exceeds Discord's limits.

## T-11. Interactive Frame key enforcement misses nested interactive components

**File:** src/page/builder.rs

render_frame_elements only identifies Button, ButtonRow, and Select as interactive when deciding whether an Element requires a stable key.

It does not identify a Section with a Button accessory. It also does not recursively inspect a Container that contains Buttons, Button Rows, or Select menus. The nested Container Frame path repeats the same limited check.

Those elements can therefore be emitted without a developer key even though they contain executions. Their custom IDs then lack the generated element identity that is supposed to keep them stable as the collection changes or paginates.

The implementation also does not reject duplicate developer keys within a resolved Frame. A repeated key combined with the same execution ID can produce the same identity for two generated elements.

**Contract not fulfilled:** Sections 5 and 9, Frame execution identity. Every interactive element generated inside a Frame must have stable identity independent of visual position.

**Test that would expose it:** Assert that unkeyed Sections with Button accessories and unkeyed Containers with interactive descendants are rejected. Assert duplicate Frame keys are rejected when they would identify distinct generated elements.

## T-12. Static and dynamic values do not share the documented direct input syntax

**Files:** src/page/value.rs, src/page/component/text.rs, src/page/component/button.rs, src/page/component/section.rs, src/page/component/container.rs

IntoValue is implemented for static values and the Dynamic wrapper. It is not implemented for an asynchronous closure itself. The public API therefore requires callers to wrap a callback with dynamic(...) before passing it to add_text, Button properties, or other IntoValue-based setters.

Page API.md demonstrates the same public method accepting a callback directly, for example add_text(|ctx, data| async move { ... }). The implementation's examples and tests instead use add_text(dynamic(...)).

The fixed Section slots have the same limitation: IntoSlot accepts Dynamic, not an arbitrary callback.

**Contract not fulfilled:** Section 2 and the design principle requiring one public interface for static and dynamic values without a separate constructor.

**Test that would expose it:** Add compile-pass examples using the exact direct-closure syntax shown in Page API.md for add_text, Button label/style, and Section slots. Those examples do not satisfy the current IntoValue/IntoSlot bounds.

## T-13. String Select exposes a disabled option setting that fails during rendering

**Files:** src/page/select/string.rs, src/page/builder.rs

SelectOption has a dynamic disabled property and a set_disabled method. During String Select rendering, the property is resolved and the render path returns an error whenever it is true.

Thus, the public builder accepts a setting that cannot be represented in the output and fails only when the Page is rendered. The API contract broadly says component properties and disabled states should resolve from static or dynamic values. If individual String Select options are intentionally not disableable, this setting should not be exposed as though it works.

**Contract/API inconsistency:** Section 2's dynamic property principle and the exposed SelectOption API.

**Test that would expose it:** Render a String Select containing a disabled option. The test should establish the intended behavior: either the option renders with the documented state, or the unsupported setter is removed/rejected at construction with an explicit contract.

## T-14. Several Modal limits are declared but not enforced

**Files:** src/page/limits.rs, src/page/modal/input.rs, src/page/modal/mod.rs

The Modal title maximum and resolved input label maximum are checked. However, MODAL_TEXT_INPUT_MAX_CHARS, MODAL_SELECT_MAX_OPTIONS, and MODAL_LABEL_DESCRIPTION_MAX_CHARS are declared but not used in the Modal render/validation path.

TextInput exposes min_length and max_length, but the render path does not validate their relationship or enforce the documented maximum. Modal option collections have no corresponding maximum check. The non-Text Modal types are not rendered at all, so their option and label constraints are not validated.

**Contract not fulfilled:** Section 12, Modal constraints, and section 10's requirement for explicit validation errors.

**Test that would expose it:** Test values exactly at and one over every documented Modal limit. Include invalid Text Input length ranges and Modal Select collections over 25 options.

## T-15. Page and Modal IDs are not unique across both registries

**Files:** src/app/registry.rs, src/app/mod.rs, src/app/store.rs, src/page/builder.rs, src/page/modal/mod.rs

PageRegistry and ModalRegistry each reject duplicates within their own map, but neither checks the other registry. The same ID can therefore be registered once as a Page and once as a Modal, despite the specification describing globally unique Page or Modal IDs.

The custom ID derivation also does not include DefinitionKind. If the Page and Modal use the same definition ID, execution ID, and serialized data, they derive the same custom ID. The StoreRecord kind then makes the second insertion conflict with the first.

**Contract not fulfilled:** Sections 1, 6, 8, and 9. Definition identity is not globally unique across Page and Modal definitions, and the derived identity is not namespaced by definition kind.

**Test that would expose it:** Register a Page and Modal with the same ID and assert that the second registration is rejected. Also derive or render records with matching IDs/data and verify they cannot collide across definition kinds.

---

# B. Issues requiring review beyond ordinary unit tests

## R-01. The custom ID digest is a pair of FNV-style 64-bit hashes

**File:** src/app/store.rs

derive_custom_id uses two 64-bit FNV-1a-style states and concatenates their hexadecimal representations. It is deterministic and the resulting custom ID is below Discord's 100-character limit. The implementation is not BLAKE3 or another established collision-resistant digest.

Page API.md gives BLAKE3 as an example of the deterministic hashing algorithm. If the purpose is only reproducible compact identifiers, the current implementation meets determinism. If the identifier is also expected to make accidental collisions infeasible, the current non-cryptographic construction is a material risk.

A finite unit test can prove that known inputs produce known outputs and that selected changed inputs produce different outputs. It cannot establish collision resistance. This requires algorithm-level review and a clear contract about the required security/collision properties.

## R-02. The tests prove isolated structures more often than working framework behavior

**Files:** src/tests/*

Several tests are useful first-principles tests, but their names/comments can make the exercised scope sound broader than it is.

Examples:

- app.rs tests the registries and App cloning, not a functional application database or interaction route.
- execution.rs proves callback types can be constructed. It does not invoke an execution through the router.
- frame.rs tests the standalone planner with simple elements. It does not prove navigation, state persistence, nested Frame pagination, or resolved dynamic budgets.
- interaction.rs tests Ping rejection and Ctx metadata. It does not test a valid Component or Modal interaction.
- modal.rs tests builder fields and a small text/boolean deserialization case. It does not prove the supported Modal input set can be rendered or submitted.
- store.rs tests MemoryStore in the same process. It does not prove durability across a process restart.
- rendering.rs proves several in-memory Serenity component structures can be produced. It does not send a Components V2 message to Discord or receive and route a resulting interaction.
- support.rs deliberately constructs App directly and uses MemoryStore. This is appropriate for a fixture, but many App tests therefore bypass the public constructor and all real external infrastructure.

The pagination_state_round_trips_through_persistence_format test is especially important to interpret correctly: it tests serde serialization of the state struct, not persistence through Store or recovery after an interaction.

The workflow's cargo test --all-targets command verifies the tests that exist. It cannot establish contracts those tests never exercise.

---

# C. Missing or underspecified contracts

These are not all straightforward implementation bugs. The specification should settle the expected behavior before the implementation is judged against it.

## G-01. Separate the host application's database from framework persistence

Page API.md says App owns the database and gives examples such as ctx.app.db.users.query(...). The code's Database is only a Store wrapper for framework interaction records.

The spec should explicitly say whether the framework owns a database for its records, receives a host application's database, or supports both as separate App fields. It should also define how an application supplies typed repositories without forcing Page data types or callbacks to depend on a concrete database implementation.

## G-02. Define Frame cardinality and nested pagination

The spec explains a Frame inside a Page and includes an example with a Frame inside a Container. It does not explicitly define whether a Page may have multiple top-level Frames, whether Frames may be nested recursively, or whether a nested Frame paginates independently or contributes to one Page-wide pagination plan.

The implementation currently rejects more than one top-level Frame and expands Container Frames inline. The intended public behavior needs to be stated.

## G-03. Define the File attachment lifecycle

The spec correctly distinguishes uploading a file from displaying a File component, but it does not fully define how uploaded bytes are associated with File::new(filename).

The implementation has RenderedMessage::add_attachment, while File only stores a filename reference. The renderer does not verify that every File reference has a matching outgoing attachment. The executable uses into_components, which intentionally discards attachments and pagination metadata.

The contract should define filename matching, duplicate filenames, missing references, whether attachment bytes are supplied before or after Page rendering, and which rendered-message conversion is safe for sending.

## G-04. Define whether every render-time property must accept callbacks

The spec gives a broad default that properties should accept static values or asynchronous callbacks, unless there is a reason not to. The implementation has properties that remain static-only, including Container spoiler behavior and Modal title. It also requires the dynamic(...) wrapper rather than accepting the closure directly.

The intended rule should identify any static-only exceptions and clarify whether the direct closure syntax in the examples is mandatory or whether an explicit dynamic adapter is an accepted part of the public API.

## G-05. Define the public rendering contract for paginated Pages

The spec promises automatic navigation, but does not define whether render returns the first message, a complete list of messages, or a handle that can produce the current page. It also does not specify the public method through which a caller sends the initial page and subsequent updates.

The current API has render and render_pages, but neither provides usable navigation. This needs an explicit public contract rather than relying on the internal PaginationState representation.

---

# D. File-by-file review inventory

Every Rust file present on the audited branch is listed below. Findings are referenced by ID above. "No independent mismatch found" means the file was reviewed and no separate contract failure was identified in that file; it does not mean its functionality is exercised end to end.

## Library and executable

- **src/lib.rs** - Exposes app and page modules and includes the test module. No independent mismatch found.
- **src/main.rs** - Demo uses MemoryStore and a private handler that does not process interactions. It sends only into_components output. See T-03, T-05, and G-03.
  
## App

- **src/app/context.rs** - RenderCtx is App-linked and temporary as specified. All interaction response methods are TODOs. See T-02.
- **src/app/database.rs** - Database wraps only Store and does not expose the application repositories shown by the contract. See T-04 and G-01.
- **src/app/error.rs** - Re-exports anyhow error/context/result types. This is consistent with the ordinary Result requirement. No independent mismatch found.
- **src/app/handler.rs** - Receives InteractionCreate but only logs that routing is not implemented. See T-03.
- **src/app/mod.rs** - App owns registries, Store-backed Database, HTTP, and router. The dispatch entry point reaches the unfinished router. See T-01 and T-04.
- **src/app/registry.rs** - Provides typed heterogeneous Page and Modal registries and rejects duplicates within each registry. It does not enforce IDs across both registries. See T-15.
- **src/app/router.rs** - Dispatches by interaction kind, but both supported route implementations are TODOs. See T-01.
- **src/app/store.rs** - Defines StoreRecord, Store, MemoryStore, and deterministic custom ID derivation. There is no durable implementation in the repository, and the digest construction needs algorithm-level review. See T-05, T-15, and R-01.

## Page builder and shared infrastructure

- **src/page/builder.rs** - Implements Page registration and much of rendering. It contains the one-top-level-Frame restriction, pagination behavior without navigation, incomplete stable-key detection, and rendering budget behavior. See T-08 through T-11.
- **src/page/execution.rs** - Defines explicit callbacks and typed selection payloads. The callback model is consistent, but the router never invokes these executions. See T-01.
- **src/page/implementation_order.rs** - Records intended implementation stages and explicitly notes unresolved Frame and inference questions. It is a planning file rather than runtime implementation. Related contract gaps are G-02 and G-04.
- **src/page/limits.rs** - Declares the documented Discord limits. Several Modal limits are not consumed by validation. See T-14.
- **src/page/mod.rs** - Exposes the Page, Component, Frame, Modal, selection, value, and render APIs. No independent mismatch found.
- **src/page/render.rs** - Defines RenderedMessage and component validation. into_components intentionally discards attachments and pagination metadata; component budgets are supplied by builder rendering and contain the omissions described in T-10. See T-10 and G-03.
- **src/page/value.rs** - Implements static and Dynamic values plus fixed slots. Raw asynchronous closures do not implement IntoValue/IntoSlot. See T-12 and G-04.

## Components

- **src/page/component/button.rs** - Button label, style, and disabled values use IntoValue and the execution is explicit. Dynamic values require the dynamic adapter. See T-12.
- **src/page/component/button_row.rs** - Uses the required five-slot tuple. Empty rows remain representable but are rejected by validation. No independent mismatch found.
- **src/page/component/container.rs** - Provides ordered component and Frame children. Nested Frame pagination is not applied by the renderer. Container spoiler is static-only. See T-09 and G-04.
- **src/page/component/file.rs** - Stores only an attachment filename reference. The separate byte attachment operation and filename association are not enforced. See G-03.
- **src/page/component/link_button.rs** - Uses a separate non-execution type and supports dynamic label/URL values. No independent mismatch found.
- **src/page/component/media_gallery.rs** - Supports item URLs, descriptions, and spoiler values and preserves item order. Gallery budget accounting differs between planning and rendering. See T-10.
- **src/page/component/mod.rs** - Defines the heterogeneous component enum and public component exports. No independent mismatch found.
- **src/page/component/section.rs** - Exposes three independent slots and Button/Thumbnail accessories. The dynamic callback adapter limitation applies to its slots. See T-12.
- **src/page/component/separator.rs** - Spacing and divider are static/dynamic values with defaults. No independent mismatch found.
- **src/page/component/text.rs** - Text Display is a distinct component and accepts IntoValue. Raw callback syntax is not accepted without dynamic(...). See T-12.
- **src/page/component/thumbnail.rs** - Thumbnail URL, description, and spoiler state support static/dynamic values and it is restricted to Section accessories. No independent mismatch found.

## Frames

- **src/page/frame/element.rs** - Provides heterogeneous Element constructors and developer keys. Enforcement of keys for nested interactive components is incomplete in the builder. See T-11.
- **src/page/frame/mod.rs** - Defines a dynamic Frame resolver and pagination opt-in. Nested/multiple Frame pagination behavior is not fully implemented or specified. See T-09 and G-02.
- **src/page/frame/pagination.rs** - Implements a standalone greedy planner and PaginationState. Budget measurement omits or miscounts component costs and dynamic values; planner output is not connected to navigation or persistence. See T-08 and T-10.

## Modals

- **src/page/modal/input.rs** - Defines all advertised Modal input variants. The render path does not implement all of them. See T-06 and T-14.
- **src/page/modal/mod.rs** - Implements Modal registration and Text Input rendering, but other input variants return errors. It persists before completing render. Some limits are not enforced. See T-06 and T-14.
- **src/page/modal/submission.rs** - Provides typed accessors and partial Serde deserialization. Entity and file values are explicitly unsupported by generic deserialization. See T-07.

## Selection menus

- **src/page/select/entity.rs** - Defines typed User, Role, Mentionable, and Channel selection callbacks. The route required to supply those values is unfinished. See T-01.
- **src/page/select/mod.rs** - Unifies the five selection menu kinds behind SelectMenu. No independent mismatch found.
- **src/page/select/string.rs** - Defines String Select options and callback shape. The exposed per-option disabled state causes rendering to fail when true. See T-13.

## Tests

- **src/tests/mod.rs** - Declares the in-tree test modules. No independent mismatch found.
- **src/tests/support.rs** - Builds an App fixture with MemoryStore and direct field construction. It is useful for isolated tests but does not exercise real persistence or gateway setup. See R-02.
- **src/tests/app.rs** - Covers App cloning, render context, constructor basics, and registry behavior. It does not test working interaction routing or application repositories. See T-01, T-04, and R-02.
- **src/tests/builder.rs** - Covers builder ordering, generic data shape, pagination opt-in, and registration typestate. It does not compile the direct raw-closure syntax in Page API.md. See T-12.
- **src/tests/components.rs** - Covers component construction, slot shape, and defaults. It does not establish complete Discord rendering or attachment association. See G-03 and R-02.
- **src/tests/execution.rs** - Covers callback type construction and selection variants, not dispatch or callback invocation. See T-01 and R-02.
- **src/tests/frame.rs** - Covers Frame construction and simple static Text Display pagination. It does not cover nested elements, dynamic budgets, navigation, or Store-backed state. See T-08 through T-11 and R-02.
- **src/tests/interaction.rs** - Covers Ping rejection and Ctx metadata only. It has no successful Component or Modal route test and no response method tests. See T-01, T-02, and R-02.
- **src/tests/modal.rs** - Covers Modal builder shape, a text accessor, and simple text/boolean Serde data. It does not render all inputs or cover typed entity/file submissions. See T-06, T-07, and R-02.
- **src/tests/rendering.rs** - Covers selected in-memory render paths, basic button persistence, and simple Frame splitting. It does not exercise Discord sending, navigation, or complete component-budget accounting. See T-08, T-10, and G-03.
- **src/tests/selection.rs** - Covers selection callback type signatures and payload variants, not rendering or interaction routing. See T-01 and R-02.
- **src/tests/store.rs** - Covers deterministic IDs and same-process MemoryStore behavior. It does not prove durability, and the identity tests do not prove collision resistance. See T-05, R-01, and R-02.
- **src/tests/validation.rs** - Covers selected static component limits. It does not cover the omitted Modal limits, complete nested message budgets, or dynamic Frame values. See T-10 and T-14.
- **src/tests/value.rs** - Covers static values, the Dynamic wrapper, and None slot conversion. It does not test raw closure conversion as shown in the API examples. See T-12.

---

## Bottom line

The most important blockers are T-01 through T-08: the interaction lifecycle, response context, handler wiring, application database surface, durable persistence, complete Modal rendering, typed submission coverage, and usable Frame navigation are not complete.

The most important test blind spot is that current passing tests can establish the correctness of isolated builders and MemoryStore operations while leaving the actual Discord interaction lifecycle entirely untested. A successful build and green unit-test run should not be treated as evidence that buttons, selections, Modals, database-backed application callbacks, or pagination work in the running bot.
