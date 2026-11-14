use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::app::{RenderCtx, store::{CustomId, derive_custom_id}};
use crate::page::{Button, ButtonStyle, Element, Page};
use super::support::{app, TestData};

#[tokio::test]
async fn frame_interactive_elements_use_their_developer_execution_id() {
    // FIRST: a Frame does not add a separate identity path or key to an
    // interactive descendant. The Page execution ID and Page data identify it.
    let app = app();
    let data = TestData::default();
    let serialized = serde_json::to_vec(&data).unwrap();
    let expected = derive_custom_id("frame-button", "open-user", &serialized).unwrap();

    let page = Page::<TestData>::new("frame-button")
        .add_frame(|_ctx, _data| async move {
            Ok(vec![Element::button(Button::new(
                "Open user",
                "open-user",
                ButtonStyle::Primary,
                Some("👤".into()),
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ))])
        })
        .register(&app)
        .unwrap();

    page.render(&app, data).await.unwrap();
    let record = app.store().resolve(&CustomId(expected.0.clone())).await.unwrap().unwrap();

    assert_eq!(record.hash, expected.0);
    assert_eq!(record.definition_id, "frame-button");
    assert_eq!(record.execution_id, "open-user");
    assert_eq!(record.serialized_data, serialized);
}

#[tokio::test]
async fn frame_interaction_identity_does_not_depend_on_element_position() {
    // FIRST: a component's callback identity comes from its developer execution
    // ID, not the transient position of the element in the Frame collection.
    let app = app();
    let data = TestData::default();
    let serialized = serde_json::to_vec(&data).unwrap();
    let expected = derive_custom_id("frame-reorder", "open-user", &serialized).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();

    let page = Page::<TestData>::new("frame-reorder")
        .add_frame(move |_ctx, _data| {
            let observed = observed.clone();
            async move {
                let button = Element::button(Button::new(
                    "Open user",
                    "open-user",
                    ButtonStyle::Primary,
                    None,
                    false,
                    |_ctx, _data: TestData| async move { Ok(()) },
                ));
                let text = Element::text("Other content");
                if observed.fetch_add(1, Ordering::SeqCst) % 2 == 0 {
                    Ok(vec![button, text])
                } else {
                    Ok(vec![text, button])
                }
            }
        })
        .register(&app)
        .unwrap();

    page.render(&app, data.clone()).await.unwrap();
    let first = app.store().resolve(&CustomId(expected.0.clone())).await.unwrap().unwrap();
    page.render(&app, data).await.unwrap();
    let second = app.store().resolve(&CustomId(expected.0.clone())).await.unwrap().unwrap();

    assert_eq!(first.hash, second.hash);
    assert_eq!(first.execution_id, "open-user");
}

#[tokio::test]
async fn duplicate_execution_ids_inside_a_frame_are_rejected() {
    // FIRST: interactive execution IDs are unique throughout the Page,
    // including controls emitted dynamically by a Frame.
    let app = app();
    let page = Page::<TestData>::new("frame-duplicate-execution")
        .add_frame(|_ctx, _data| async move {
            let first = Element::button(Button::new(
                "First",
                "duplicate",
                ButtonStyle::Primary,
                None,
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ));
            let second = Element::button(Button::new(
                "Second",
                "duplicate",
                ButtonStyle::Secondary,
                Some("⚠️".into()),
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ));
            Ok(vec![first, second])
        })
        .register(&app)
        .unwrap();

    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn frame_can_resolve_dynamic_component_properties_in_its_elements() {
    // FIRST: Frame elements use the same static-or-dynamic property conversion
    // as components declared directly on the Page.
    let app = app();
    let page = Page::<TestData>::new("frame-dynamic-property")
        .add_frame(|_ctx: RenderCtx, data: Arc<TestData>| async move {
            Ok(vec![Element::text(
                move |_ctx: RenderCtx, data: Arc<TestData>| async move {
                    Ok(format!("{}:{}", data.label, data.count))
                },
            )])
        })
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();
    assert!(json.to_string().contains("sample:7"));
}


