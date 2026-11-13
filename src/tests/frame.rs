use std::sync::Arc;
use crate::app::RenderCtx;
use crate::page::frame::pagination::{plan, ComponentBudget, PaginationState};
use crate::page::{Element, Frame};
use super::support::{app, TestData};

#[test]
fn frame_elements_support_heterogeneous_component_construction_and_stable_keys() {
    // First-principles check: Frame output is heterogeneous and interactive
    // identity comes from a developer key, not the current vector index.
    let element = Element::<TestData>::text("User row").keyed("user-42");
    assert_eq!(element.key.as_deref(), Some("user-42"));
}

#[tokio::test]
async fn frame_resolver_receives_render_context_and_shared_page_data() {
    // First-principles check: dynamic collections get the same App access and T
    // available to ordinary dynamic properties.
    let app = app();
    let frame = Frame::<TestData>::new("user-rows", |ctx: RenderCtx, data: Arc<TestData>| async move {
        let _db = ctx.app.db.clone();
        Ok(vec![Element::text(data.label.clone()).keyed("first")])
    });
    let elements = (frame.resolver)(app.render_context("frame-page"), Arc::new(TestData::default()))
        .await
        .unwrap();
    assert_eq!(elements.len(), 1);
    assert_eq!(elements[0].key.as_deref(), Some("first"));
}

#[test]
fn frame_pagination_is_opt_in_and_builder_fluent() {
    // First-principles check: this verifies the explicit opt-in API, not the paging algorithm.
    let frame = Frame::<TestData>::new("users", |_ctx, _data| async move {
        Ok(Vec::<Element<TestData>>::new())
    }).paginate();
    assert!(frame.paginate);
}

#[test]
fn pagination_splits_only_between_complete_elements_and_preserves_order() {
    // First-principles check: this tests user-visible ordering and completeness,
    // not the internal bin-packing strategy.
    let elements = (0..41)
        .map(|index| Element::<TestData>::text(format!("Row {index}")).keyed(format!("row-{index}")))
        .collect();
    let plan = plan("rows", elements, ComponentBudget::default(), ComponentBudget::default()).unwrap();

    assert_eq!(plan.pages.len(), 2);
    assert_eq!(plan.pages[0].len(), 40);
    assert_eq!(plan.pages[1].len(), 1);
    assert_eq!(plan.pages[0][0].key.as_deref(), Some("row-0"));
    assert_eq!(plan.pages[1][0].key.as_deref(), Some("row-40"));
}

#[test]
fn pagination_reserves_static_and_navigation_budgets_before_filling_pages() {
    // First-principles check: static content and navigation consume the same
    // component capacity as Frame elements; the Frame must maximize the remainder.
    let elements = (0..40)
        .map(|index| Element::<TestData>::text(format!("Row {index}")))
        .collect();
    let plan = plan(
        "rows",
        elements,
        ComponentBudget { total_components: 2, text_display_chars: 100, direct_container_children: 0 },
        ComponentBudget { total_components: 1, text_display_chars: 0, direct_container_children: 0 },
    ).unwrap();

    assert_eq!(plan.pages[0].len(), 37);
    assert_eq!(plan.pages[1].len(), 3);
}

#[test]
fn pagination_rejects_a_single_element_that_cannot_fit() {
    // First-principles check: an indivisible oversized element must return an
    // error rather than being truncated to fit a page.
    let elements = vec![Element::<TestData>::text("x".repeat(4_001))];
    assert!(plan("oversized", elements, ComponentBudget::default(), ComponentBudget::default()).is_err());
}


#[test]
fn pagination_respects_the_shared_text_display_character_budget() {
    // First-principles check: Text Displays share one message-wide character
    // budget. The Frame must move a complete next element to the following page.
    let elements = (0..5)
        .map(|index| Element::<TestData>::text(format!("{index}{}", "x".repeat(999))))
        .collect();
    let plan = plan("long-text", elements, ComponentBudget::default(), ComponentBudget::default()).unwrap();

    assert_eq!(plan.pages.len(), 2);
    assert_eq!(plan.pages[0].len(), 4);
    assert_eq!(plan.pages[1].len(), 1);
}

#[test]
fn pagination_state_round_trips_through_persistence_format() {
    // First-principles check: the current Frame page must be recoverable after
    // the interaction or process that created it is gone.
    let state = PaginationState { frame_id: "users".into(), page_index: 2, page_count: 5 };
    let encoded = serde_json::to_vec(&state).unwrap();
    let decoded: PaginationState = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, state);
}
