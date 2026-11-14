use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{
    app::{RenderCtx, store::{CustomId, derive_custom_id}},
    page::{
        Button, ButtonStyle, Container, Element, LinkButton, Modal, Page, Separator, TextInputStyle,
    },
};
use super::support::{app, TestData};

#[tokio::test]
async fn page_render_renders_container_children_in_order() {
    let app = app();
    let page = Page::<TestData>::new("render-container")
        .add_container(
            Container::new()
                .set_accent_color(0x5865F2)
                .add_text("Container heading")
                .add_separator(Separator::new())
                .add_text("Container footer"),
        )
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();

    assert_eq!(rendered.components().len(), 1);
    let (components, attachments, _) = rendered.into_parts();
    assert_eq!(components.len(), 1);
    assert!(attachments.is_empty());
    let serialized = serde_json::to_value(&components[0]).unwrap();
    assert_eq!(serialized["type"], 17);
    assert_eq!(serialized["components"].as_array().unwrap().len(), 3);
    assert_eq!(serialized["components"][0]["content"], "Container heading");
    assert_eq!(serialized["components"][1]["type"], separator_type());
    assert_eq!(serialized["components"][2]["content"], "Container footer");
}

fn separator_type() -> u64 { 14 }

#[tokio::test]
async fn page_render_resolves_dynamic_values_with_page_data_and_app_context() {
    let app = app();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();

    let page = Page::<TestData>::new("render-dynamic")
        .add_text(
            move |ctx: RenderCtx, data: Arc<TestData>| {
                let observed = observed.clone();
                async move {
                    assert_eq!(ctx.definition_id.as_ref(), "render-dynamic");
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(format!("{}:{}", data.label, data.count))
                }
            },
        )
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(rendered.components().len(), 1);
}

#[tokio::test]
async fn page_render_preserves_dynamic_callback_errors_and_adds_page_context() {
    let app = app();
    let page = Page::<TestData>::new("render-error")
        .add_text(
            |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                Err(anyhow::anyhow!("the dynamic source failed"))
            },
        )
        .register(&app)
        .unwrap();

    let error = page.render(&app, TestData::default()).await.err()
        .expect("this render is expected to fail");
    let chain = format!("{error:#}");

    assert!(chain.contains("the dynamic source failed"));
    assert!(chain.contains("render-error"));
}

#[tokio::test]
async fn page_render_persists_button_execution_identity_and_page_data() {
    let app = app();
    let data = TestData::default();
    let serialized_data = serde_json::to_vec(&data).unwrap();
    let expected_id = derive_custom_id(
        "render-persist",
        "open-profile",
        &serialized_data,
    )
    .unwrap();

    let page = Page::<TestData>::new("render-persist")
        .add_button_row((
            Button::new(
                "Open profile",
                "open-profile",
                ButtonStyle::Primary,
                Some("👤".into()),
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ),
            None,
            None,
            None,
            None,
        ))
        .register(&app)
        .unwrap();

    let _rendered = page.render(&app, data).await.unwrap();
    let record = app.store()
        .resolve(&expected_id)
        .await
        .unwrap()
        .expect("render must persist its interactive execution");

    assert_eq!(record.hash, expected_id.0);
    assert_eq!(record.definition_id, "render-persist");
    assert_eq!(record.execution_id, "open-profile");
    assert_eq!(record.serialized_data, serialized_data);
}

#[tokio::test]
async fn link_buttons_render_without_routing_records() {
    // FIRST: Link Buttons are URL components, not callback executions. A
    // sibling callback Button still receives its ordinary deterministic route.
    let app = app();
    let data = TestData::default();
    let serialized = serde_json::to_vec(&data).unwrap();
    let callback_id = derive_custom_id("link-and-callback", "refresh", &serialized).unwrap();

    let page = Page::<TestData>::new("link-and-callback")
        .add_button_row((
            Button::new(
                "Refresh",
                "refresh",
                ButtonStyle::Secondary,
                None,
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ),
            LinkButton::new(
                "Documentation",
                "https://example.test/docs",
                Some("🌐".into()),
                true,
            ),
            None,
            None,
            None,
        ))
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, data).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();

    fn find_label<'a>(value: &'a serde_json::Value, label: &str) -> Option<&'a serde_json::Value> {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("label").and_then(serde_json::Value::as_str) == Some(label) {
                    return Some(value);
                }
                map.values().find_map(|child| find_label(child, label))
            }
            serde_json::Value::Array(items) => items.iter().find_map(|child| find_label(child, label)),
            _ => None,
        }
    }

    fn collect_custom_ids(value: &serde_json::Value, ids: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                if let Some(id) = map.get("custom_id").and_then(serde_json::Value::as_str) {
                    ids.push(id.to_owned());
                }
                for child in map.values() {
                    collect_custom_ids(child, ids);
                }
            }
            serde_json::Value::Array(items) => {
                for child in items {
                    collect_custom_ids(child, ids);
                }
            }
            _ => {}
        }
    }

    let link = find_label(&json, "Documentation").expect("Link Button must be rendered");
    assert_eq!(link["url"], "https://example.test/docs");
    assert_eq!(link["disabled"], true);
    assert!(link.get("custom_id").is_none(), "Link Button must not have a custom_id");

    let callback = find_label(&json, "Refresh").expect("callback Button must be rendered");
    assert_eq!(callback["custom_id"], callback_id.0);

    let mut ids = Vec::new();
    collect_custom_ids(&json, &mut ids);
    assert_eq!(ids, vec![callback_id.0.clone()]);

    let record = app.store().resolve(&callback_id).await.unwrap()
        .expect("callback Button must have a routing record");
    assert_eq!(record.execution_id, "refresh");
}

#[tokio::test]
async fn page_render_expands_frame_at_its_declared_position_once() {
    let app = app();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();

    let page = Page::<TestData>::new("render-frame")
        .add_text("Static header")
        .add_frame(move |ctx: RenderCtx, data: Arc<TestData>| {
            let observed = observed.clone();
            async move {
                assert_eq!(ctx.definition_id.as_ref(), "render-frame");
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(vec![
                    Element::text(format!("{}-first", data.label)),
                    Element::text(format!("{}-second", data.label)),
                ])
            }
        })
        .add_text("Static footer")
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();
    let header = json.find("Static header").unwrap();
    let first = json.find("sample-first").unwrap();
    let second = json.find("sample-second").unwrap();
    let footer = json.find("Static footer").unwrap();
    assert!(header < first && first < second && second < footer);
}

#[tokio::test]
async fn modal_render_persists_modal_data_and_returns_a_modal_response() {
    let app = app();
    let data = TestData::default();
    let serialized_data = serde_json::to_vec(&data).unwrap();
    let expected_id = derive_custom_id("render-modal", "submit", &serialized_data).unwrap();

    let modal = Modal::<TestData>::new("render-modal")
        .set_title("Edit profile")
        .add_text_input("username", "Username", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission| async move { Ok(()) })
        .register(&app)
        .unwrap();

    let rendered = modal.render(&app, data).await.unwrap();

    assert_eq!(rendered.custom_id, expected_id.0);
    let record = app.store()
        .resolve(&CustomId(rendered.custom_id.clone()))
        .await
        .unwrap()
        .expect("rendering a Modal must persist its submit execution");
    assert_eq!(record.hash, rendered.custom_id);
    assert_eq!(record.definition_id, "render-modal");
    assert_eq!(record.execution_id, "submit");
    assert_eq!(record.serialized_data, serialized_data);
}

#[test]
fn page_registration_rejects_duplicate_execution_ids() {
    let app = app();
    let page = Page::<TestData>::new("duplicate-executions")
        .add_button_row((
            Button::new(
                "First",
                "duplicate",
                ButtonStyle::Primary,
                None,
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ),
            Button::new(
                "Second",
                "duplicate",
                ButtonStyle::Secondary,
                Some("⚠️".into()),
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ),
            None,
            None,
            None,
        ));

    assert!(page.register(&app).is_err());
}


