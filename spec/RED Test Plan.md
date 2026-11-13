# RED Phase Test Plan

Basis: `spec/Page API.md` and the architecture notes in `spec/Robo-Pope.canvas`.
The spec defines the required behavior; the source comments describe the contract
at the public API boundary. The tests live in `src/tests/`, split by API area,
and are included as crate unit tests.

## Test-first rules

A test is retained when it protects a developer-visible guarantee, a boundary
condition, or a contract between two parts of the framework. Each test has a
nearby `First-principles check` comment explaining why it earns its place.

I excluded tests that only repeat a constant, snapshot Serenity builder internals,
depend on live Discord/network behavior, or assert implementation details with no
user-facing consequence. Tests that inspect our own Page/component description
state are limited to guarantees such as ordering and fixed slots; those checks
should later be complemented by rendered-output tests.

## Phase 1: contracts added

Concise Rustdoc contracts were added alongside the App, Ctx, RenderCtx,
registries, Store, Page, Component, Execution, Select, Modal, Frame, pagination,
render output, and value-conversion declarations. They describe what callers
may rely on, including identity, ownership, serialization, callback inputs,
validation, error behavior, and limits. They do not describe unimplemented
internals as though they already work.

## Phase 2: RED unit tests

The suite is intentionally split by responsibility:

### App and registries: `src/tests/app.rs`

- `app_clone_shares_the_same_owned_database`: retained because cloned contexts must share App-owned state.
- `render_context_points_back_to_the_same_app_state`: retained because RenderCtx must link to the same App.
- `app_constructor_initializes_empty_registries_and_keeps_database`: retained because it tests the public construction path.
- `page_registry_holds_multiple_page_data_types`: retained because one App must hold heterogeneous Page definitions.
- `page_registry_rejects_duplicate_definition_ids`: retained because IDs identify immutable definitions.
- `page_registry_rejects_a_query_using_the_wrong_data_type`: retained because type erasure must not leak into invalid typed handles.
- `registry_reports_unknown_ids_as_errors`: retained because missing definitions must be explicit errors.
- `modal_registry_reports_unknown_definition_ids_as_errors`: retained because Modal lookup has the same missing-ID contract.

### Page builder: `src/tests/builder.rs`

- `page_builder_preserves_top_level_component_order`: retained because composition order is visible in the resulting message; this unit test inspects the builder's ordered representation until render tests exist.
- `page_data_type_is_shared_by_static_and_dynamic_components`: retained because every component in a Page must use the same T and can access App through RenderCtx.
- `paginate_is_opt_in_and_keeps_the_page_builder_chain`: retained because this checks a direct builder option, not pagination internals.
- `registering_a_page_returns_an_immutable_registered_handle`: retained because registration is the public typestate boundary.

### Components: `src/tests/components.rs`

- `section_supports_three_independent_text_slots_and_empty_slots`: retained because Section's fixed three-slot shape is a structural contract.
- `section_accepts_a_thumbnail_as_its_single_accessory`: retained because this proves the supported composition path.
- `button_row_has_five_explicit_optional_slots`: retained because the tuple prevents a sixth Button slot.
- `container_preserves_the_declared_order_of_mixed_children`: retained because mixed child ordering is visible and specified.
- `media_gallery_keeps_items_in_the_order_added`: retained because gallery item order is visible.
- `button_callback_receives_page_data_and_app_linked_context`: retained because it compiles the intended callback use with T and Ctx.
- `link_button_is_configured_with_a_url_without_an_execution`: retained because URL navigation is distinct from callback execution.
- `file_component_references_an_outgoing_attachment_by_filename`: retained because File references attachment bytes supplied separately.
- `text_display_accepts_a_dynamic_content_value`: retained because static and dynamic text share one constructor.
- `separator_defaults_to_small_spacing_and_visible_divider`: retained because these defaults are observable API behavior.

### Executions: `src/tests/execution.rs`

- `button_execution_has_a_stable_developer_defined_id`: retained because execution IDs are used for routing.
- `typed_execution_accepts_an_interaction_payload_separate_from_page_data`: retained because interaction input and serialized Page data are different values.
- `selection_keeps_each_supported_payload_kind_distinct`: retained because selection kind must remain typed.

### Frames and pagination: `src/tests/frame.rs`

- `frame_elements_support_heterogeneous_component_construction_and_stable_keys`: retained because heterogeneous output and stable keys are core Frame guarantees.
- `frame_resolver_receives_render_context_and_shared_page_data`: retained because Frame callbacks need the same App-linked context and T. It directly invokes the resolver as a unit-level check; later Page-render tests should cover the public route.
- `frame_pagination_is_opt_in_and_builder_fluent`: retained because it checks the explicit opt-in.
- `pagination_splits_only_between_complete_elements_and_preserves_order`: retained because pagination must preserve complete components and their order.
- `pagination_reserves_static_and_navigation_budgets_before_filling_pages`: retained because fixed Page content and framework navigation consume message capacity too.
- `pagination_rejects_a_single_element_that_cannot_fit`: retained because oversized indivisible content must return an error, not be truncated.
- `pagination_respects_the_shared_text_display_character_budget`: retained because Text Display content shares one message-wide budget.
- `pagination_state_round_trips_through_persistence_format`: retained because the current Frame page must be recoverable.

### Modals and submissions: `src/tests/modal.rs`

- `modal_builder_starts_with_its_developer_defined_id_and_title`: retained because ID and title are direct Modal API inputs.
- `modal_input_ids_are_the_keys_used_for_submitted_values`: retained because stable input IDs connect definitions to submitted fields.
- `modal_raw_submit_callback_receives_context_data_and_submission_separately`: retained because original Modal data and submitted values are distinct.
- `modal_typed_submit_callback_uses_serde_form_data`: retained because the typed API must accept a developer-defined Serde struct.
- `modal_registration_returns_a_typed_registered_handle`: retained because registration freezes the Modal definition.
- `submission_text_accessor_returns_the_value_for_its_input_id`: retained because callers retrieve fields by their assigned IDs.
- `submission_accessor_rejects_missing_and_wrong_type_values`: retained because invalid input must be an error, not coerced or invented.
- `typed_submission_deserializes_by_input_id`: retained because this is the documented ergonomic Serde path.
- `modal_options_keep_label_and_submitted_value_independent`: retained because presentation labels must not replace submitted values.

### Selection menus: `src/tests/selection.rs`

- `string_select_option_keeps_display_label_separate_from_submitted_value`: retained because callback values are stable identifiers, not labels.
- `string_select_callback_receives_selected_values_in_a_typed_vector`: retained because the callback receives a typed collection separate from Page data.
- `entity_select_callbacks_receive_their_specific_selected_type`: retained because User, Role, Mentionable, and Channel selections are different payloads.
- `selection_payload_variants_preserve_the_menu_kind`: retained because callers must be able to distinguish selection kinds.

### Persistence: `src/tests/store.rs`

- `custom_id_derivation_is_deterministic_for_the_same_execution_identity`: retained because repeated renders of the same identity must reuse a custom ID.
- `custom_id_changes_when_any_identity_input_changes`: retained because different definitions, executions, data, or Frame keys must not collide by design.
- `memory_store_round_trips_a_record_by_custom_id`: retained because routing requires restoring the definition, execution, and serialized data.
- `inserting_the_same_record_twice_is_idempotent`: retained because identical rerenders should reuse an existing record.
- `store_rejects_conflicting_records_for_the_same_custom_id`: retained because one custom ID cannot safely resolve to conflicting callback data.
- `removing_a_missing_record_is_harmless`: retained because cleanup should be repeatable.

### Static and dynamic values: `src/tests/value.rs`

- `static_values_convert_without_a_special_constructor`: retained because ordinary Rust values are the intended public input.
- `supported_primitive_and_enum_values_convert_to_static_values`: retained because supported property types should use their natural values.
- `dynamic_values_resolve_with_page_data_and_render_context`: retained because this is the central static/dynamic property contract.
- `dynamic_value_errors_are_returned_without_being_replaced`: retained because callers must receive the original error.
- `optional_slot_converts_none_to_an_empty_slot`: retained because empty fixed slots should use ordinary Option syntax.
- `optional_slot_converts_static_and_dynamic_values`: retained because fixed slots follow the same value conversion rules.

### Discord validation: `src/tests/validation.rs`

- `validator_accepts_a_text_display_at_the_character_limit`: retained because the exact documented boundary must be accepted.
- `validator_rejects_a_text_display_over_the_character_limit`: retained because invalid output must not be silently truncated.
- `validator_rejects_a_container_with_too_many_direct_children`: retained because Container child count is a Discord constraint.
- `validator_rejects_a_section_without_any_text_display`: retained because a Section needs at least one Text Display.
- `validator_rejects_an_empty_button_row`: retained because an Action Row needs at least one Button.
- `validator_rejects_a_media_gallery_with_more_than_ten_items`: retained because the Media Gallery has a fixed maximum.

## Known API gap

The spec and canvas show application repositories being accessed through the
App, for example `ctx.app.db.users`. The current `Database` declaration only
models the framework's execution Store. The contract comments now call this out.
No repository trait or `users` API is invented here; the App-owned database
shape must be defined before that example can be considered compile-ready.

## Expected RED state

Many behavior methods are still `todo!()`. Tests that call those methods are
expected to fail until the corresponding implementation is written. The first
check should be `cargo test --no-run` to distinguish API/test compilation
problems from the expected runtime RED failures. The green implementation phase
is deliberately not part of this change.
