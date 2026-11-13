use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use crate::app::RenderCtx;
use crate::app::store::derive_custom_id;
use crate::page::{Container, Element, Frame, Page};
use super::support::{app, component_interaction, TestData};

fn find_button<'a>(value: &'a serde_json::Value, label: &str) -> Option<&'a serde_json::Value> {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("label").and_then(serde_json::Value::as_str) == Some(label) {
                return Some(value);
            }
            map.values().find_map(|child| find_button(child, label))
        }
        serde_json::Value::Array(items) => items.iter().find_map(|child| find_button(child, label)),
        _ => None,
    }
}

fn assert_container_child_limits(value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(serde_json::Value::as_u64) == Some(17) {
                let children = map.get("components").and_then(serde_json::Value::as_array)
                    .expect("Container must serialize its direct children");
                assert!(children.len() <= 10, "Container direct-child limit exceeded");
            }
            for child in map.values() { assert_container_child_limits(child); }
        }
        serde_json::Value::Array(items) => {
            for child in items { assert_container_child_limits(child); }
        }
        _ => {}
    }
}

#[tokio::test]
async fn paginated_frame_renders_framework_previous_and_next_controls() {
    // FIRST: pagination is framework-managed; controls are visible on each page.
    let app = app();
    let page = Page::<TestData>::new("pagination-controls")
        .add_frame(Frame::<TestData>::new("rows", |_ctx, _data| async move {
            Ok((0..41).map(|n| Element::text(format!("Row {n}")).keyed(format!("row-{n}"))).collect())
        }).paginate()).register(&app).unwrap();
    let rendered = page.render_pages(&app, TestData::default()).await.unwrap();
    assert_eq!(rendered.pages.len(), 2);
    for (index, message) in rendered.pages.into_iter().enumerate() {
        let (components, _, state) = message.into_parts();
        let json = serde_json::to_value(components).unwrap();
        let previous = find_button(&json, "Previous").expect("Previous control must exist");
        let next = find_button(&json, "Next").expect("Next control must exist");
        assert_eq!(previous["disabled"].as_bool(), Some(index == 0));
        assert_eq!(next["disabled"].as_bool(), Some(index == 1));
        assert!(state.is_some());
    }
}

#[tokio::test]
async fn a_single_page_frame_has_no_navigation_controls() {
    // FIRST: controls exist only when there is another page to navigate to.
    let app = app();
    let page = Page::<TestData>::new("pagination-single-page")
        .add_frame(Frame::<TestData>::new("small", |_ctx, _data| async move {
            Ok(vec![Element::text("One").keyed("one"), Element::text("Two").keyed("two")])
        }).paginate()).register(&app).unwrap();
    let rendered = page.render_pages(&app, TestData::default()).await.unwrap();
    assert_eq!(rendered.pages.len(), 1);
    let (components, _, _) = rendered.pages.into_iter().next().unwrap().into_parts();
    let json = serde_json::to_value(components).unwrap();
    assert!(find_button(&json, "Previous").is_none());
    assert!(find_button(&json, "Next").is_none());
}

#[tokio::test]
async fn pagination_navigation_records_persist_frame_identity_and_current_position() {
    // FIRST: navigation records retain Page, Frame, original data, and current index.
    let app = app();
    let data = TestData::default();
    let serialized = serde_json::to_vec(&data).unwrap();
    let page = Page::<TestData>::new("pagination-record")
        .add_frame(Frame::<TestData>::new("users", |_ctx, _data| async move {
            Ok((0..41).map(|n| Element::text(format!("User {n}")).keyed(format!("user-{n}"))).collect())
        }).paginate()).register(&app).unwrap();
    let _ = page.render_pages(&app, data).await.unwrap();
    let previous = derive_custom_id("pagination-record", "__frame:users:previous", &serialized, Some("users")).unwrap();
    let next = derive_custom_id("pagination-record", "__frame:users:next", &serialized, Some("users")).unwrap();
    let previous_record = app.store().resolve(&previous).await.unwrap().unwrap();
    let next_record = app.store().resolve(&next).await.unwrap().unwrap();
    assert_eq!(previous_record.pagination_state.as_ref().unwrap()["page_index"], 0);
    assert_eq!(next_record.pagination_state.as_ref().unwrap()["page_index"], 0);
    assert_eq!(previous_record.definition_id, "pagination-record");
    assert_eq!(next_record.execution_id, "__frame:users:next");
}

#[tokio::test]
async fn next_and_previous_navigation_update_state_and_reject_out_of_range_clicks() {
    // FIRST: navigation advances one position, rejects stale clicks, and never escapes bounds.
    let app = app();
    let data = TestData::default();
    let serialized = serde_json::to_vec(&data).unwrap();
    let page = Page::<TestData>::new("pagination-boundaries")
        .add_frame(Frame::<TestData>::new("items", |_ctx, _data| async move {
            Ok((0..41).map(|n| Element::text(format!("Item {n}")).keyed(format!("item-{n}"))).collect())
        }).paginate()).register(&app).unwrap();
    let _ = page.render_pages(&app, data).await.unwrap();
    let previous = derive_custom_id("pagination-boundaries", "__frame:items:previous", &serialized, Some("items")).unwrap();
    let next = derive_custom_id("pagination-boundaries", "__frame:items:next", &serialized, Some("items")).unwrap();
    assert!(app.dispatch_interaction(component_interaction(&previous.0)).await.is_err());
    app.dispatch_interaction(component_interaction(&next.0)).await.unwrap();
    let advanced = app.store().resolve(&next).await.unwrap().unwrap();
    assert_eq!(advanced.pagination_state.as_ref().unwrap()["page_index"], 1);
    assert!(app.dispatch_interaction(component_interaction(&next.0)).await.is_err());
    let final_state = app.store().resolve(&next).await.unwrap().unwrap();
    assert_eq!(final_state.pagination_state.as_ref().unwrap()["page_index"], 1);
}

#[tokio::test]
async fn nested_paginated_frame_splits_inside_its_container_and_repeats_static_children() {
    // FIRST: a nested Frame paginates in place while Container siblings repeat.
    let app = app();
    let container = Container::<TestData>::new().add_text("Container header")
        .add_frame(Frame::<TestData>::new("nested-rows", |_ctx, _data| async move {
            Ok((0..15).map(|n| Element::text(format!("Nested {n}")).keyed(format!("nested-{n}"))).collect())
        }).paginate()).add_text("Container footer");
    let page = Page::<TestData>::new("nested-frame").add_container(container).register(&app).unwrap();
    let rendered = page.render_pages(&app, TestData::default()).await.unwrap();
    assert!(rendered.pages.len() > 1);
    for message in rendered.pages {
        let (components, _, _) = message.into_parts();
        let json = serde_json::to_value(components).unwrap();
        assert!(json.to_string().contains("Container header"));
        assert!(json.to_string().contains("Container footer"));
        assert_container_child_limits(&json);
    }
}

#[tokio::test]
async fn page_rejects_more_than_one_paginated_frame_across_nested_components() {
    // FIRST: a Page has one unambiguous pagination position and state.
    let app = app();
    let nested = Container::<TestData>::new().add_frame(Frame::<TestData>::new("nested", |_ctx, _data| async move {
        Ok(vec![Element::text("Nested").keyed("nested")])
    }).paginate());
    let page = Page::<TestData>::new("multiple-frames")
        .add_frame(Frame::<TestData>::new("top", |_ctx, _data| async move {
            Ok(vec![Element::text("Top").keyed("top")])
        }).paginate()).add_container(nested);
    match page.register(&app) {
        Err(_) => {}
        Ok(registered) => assert!(registered.render_pages(&app, TestData::default()).await.is_err()),
    }
}

#[tokio::test]
async fn dynamic_frame_properties_are_counted_before_page_boundaries_are_chosen() {
    // FIRST: resolved dynamic property output consumes the message-wide text budget.
    let app = app();
    let page = Page::<TestData>::new("dynamic-frame-budget")
        .add_frame(Frame::<TestData>::new("dynamic", |_ctx, _data| async move {
            Ok((0..20).map(|n| Element::text(crate::page::dynamic(
                move |_ctx: RenderCtx, data: Arc<TestData>| async move {
                    Ok(format!("{}-{n}-{}", data.label, "x".repeat(205)))
                },
            )).keyed(format!("line-{n}"))).collect())
        }).paginate()).register(&app).unwrap();
    let rendered = page.render_pages(&app, TestData::default()).await.unwrap();
    assert!(rendered.pages.len() >= 2);
}

#[tokio::test]
async fn nested_section_accessory_requires_a_stable_frame_key() {
    // FIRST: a Section accessory Button is an interactive Frame descendant.
    let app = app();
    let page = Page::<TestData>::new("nested-key-section")
        .add_frame(Frame::<TestData>::new("f", |_ctx, _data| async move {
            Ok(vec![Element::section(crate::page::Section::new().set_text("Info", None, None)
                .set_accessory(crate::page::Button::new("Act", "section-action", crate::page::ButtonStyle::Primary,
                    |_ctx, _data: TestData| async move { Ok(()) })))])
        })).register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn nested_container_button_requires_a_stable_frame_key() {
    // FIRST: a Button inside a Container is an interactive Frame descendant.
    let app = app();
    let page = Page::<TestData>::new("nested-key-container")
        .add_frame(Frame::<TestData>::new("f", |_ctx, _data| async move {
            Ok(vec![Element::container(Container::new().add_button(crate::page::Button::new(
                "Nested", "container-action", crate::page::ButtonStyle::Secondary,
                |_ctx, _data: TestData| async move { Ok(()) },
            )))])
        })).register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn stable_frame_key_preserves_custom_id_when_elements_reorder() {
    // FIRST: stable developer keys, not vector positions, determine identity.
    let app = app();
    let order = Arc::new(AtomicUsize::new(0));
    let callback_order = order.clone();
    let page = Page::<TestData>::new("stable-nested-key")
        .add_frame(Frame::<TestData>::new("users", move |_ctx, _data| {
            let call = callback_order.fetch_add(1, Ordering::SeqCst);
            async move {
                let user = Element::container(Container::new().add_button(crate::page::Button::new(
                    "Open", "open", crate::page::ButtonStyle::Primary,
                    |_ctx, _data: TestData| async move { Ok(()) },
                ))).keyed("user-42");
                let other = Element::text("Other user").keyed("user-43");
                Ok(if call % 2 == 0 { vec![user, other] } else { vec![other, user] })
            }
        })).register(&app).unwrap();
    let data = TestData::default();
    let _ = page.render(&app, data.clone()).await.unwrap();
    let first = derive_custom_id("stable-nested-key", "open", &serde_json::to_vec(&data).unwrap(), Some("users:user-42")).unwrap();
    let _ = page.render(&app, data.clone()).await.unwrap();
    let second = derive_custom_id("stable-nested-key", "open", &serde_json::to_vec(&data).unwrap(), Some("users:user-42")).unwrap();
    assert_eq!(first, second);
    let record = app.store().resolve(&first).await.unwrap().unwrap();
    assert_eq!(record.frame_identity.as_deref(), Some("users:user-42"));
}

#[tokio::test]
async fn frame_resolver_is_not_reinvoked_to_recalculate_a_page_plan() {
    // FIRST: one render uses one resolved Frame collection for planning and output.
    let app = app();
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    let page = Page::<TestData>::new("frame-once")
        .add_frame(Frame::<TestData>::new("rows", move |_ctx: RenderCtx, _data| {
            let seen = seen.clone();
            async move {
                seen.fetch_add(1, Ordering::SeqCst);
                Ok((0..41).map(|n| Element::text(format!("Row {n}")).keyed(format!("row-{n}"))).collect())
            }
        }).paginate()).register(&app).unwrap();
    let _ = page.render_pages(&app, TestData::default()).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
