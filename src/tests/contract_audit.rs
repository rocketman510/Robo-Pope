//! RED-phase contract tests derived from `spec/Rust Implementation Audit.md`
//! and cross-checked against `spec/Page API.md`.
//!
//! These tests are expected to FAIL until the corresponding contract surface is implemented.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Deserialize;
use serenity::all::Interaction;

use crate::app::{Ctx, Database};
use crate::app::store::{CustomId, DefinitionKind, MemoryStore, Store, StoreRecord};
use crate::page::{
    Button, ButtonStyle, Container, Element, File, Frame, Modal, ModalInput,
    ModalOption, ModalSubmission, Page, Section, SelectOption,
    StringSelect, SubmittedValue, TextInputStyle, UserSelect,
};
use crate::page::modal::input::TextInput;
use crate::page::value::Value;
use super::support::{app, TestData};

fn ping_interaction() -> Interaction {
    serde_json::from_str(
        r#"{
            "type": 1,
            "id": "100",
            "application_id": "200",
            "token": "test-interaction-token",
            "version": 1,
            "app_permissions": "0"
        }"#,
    )
    .expect("minimal Ping interaction fixture must deserialize")
}

fn component_interaction(custom_id: &str) -> Interaction {
    let payload = format!(
        r#"{{
            "type": 3,
            "id": "9001",
            "application_id": "200",
            "token": "component-token",
            "version": 1,
            "app_permissions": "0",
            "channel_id": "300",
            "user": {{
                "id": "400",
                "username": "tester",
                "discriminator": "0",
                "global_name": null,
                "avatar": null
            }},
            "message": {{
                "id": "500",
                "channel_id": "300",
                "author": {{
                    "id": "200",
                    "username": "bot",
                    "discriminator": "0",
                    "global_name": null,
                    "avatar": null
                }},
                "content": "",
                "timestamp": "2024-01-01T00:00:00.000000+00:00",
                "edited_timestamp": null,
                "tts": false,
                "mention_everyone": false,
                "mentions": [],
                "mention_roles": [],
                "attachments": [],
                "embeds": [],
                "pinned": false,
                "type": 0,
                "flags": 0,
                "components": []
            }},
            "data": {{
                "custom_id": "{custom_id}",
                "component_type": 2
            }}
        }}"#
    );
    serde_json::from_str(&payload).expect("component interaction fixture must deserialize")
}

fn modal_submit_interaction(custom_id: &str) -> Interaction {
    let payload = format!(
        r#"{{
            "type": 5,
            "id": "9002",
            "application_id": "200",
            "token": "modal-token",
            "version": 1,
            "app_permissions": "0",
            "channel_id": "300",
            "user": {{
                "id": "400",
                "username": "tester",
                "discriminator": "0",
                "global_name": null,
                "avatar": null
            }},
            "data": {{
                "custom_id": "{custom_id}",
                "components": []
            }}
        }}"#
    );
    serde_json::from_str(&payload).expect("modal submit fixture must deserialize")
}

#[tokio::test]
async fn t01_component_route_resolves_store_or_returns_clean_error() {
    let app = app();
    let _page = Page::<TestData>::new("t01-page")
        .add_button(Button::new("Go", "t01-btn", ButtonStyle::Primary, |_ctx, _data: TestData| async move { Ok(()) }))
        .register(&app).unwrap();
    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = crate::app::store::derive_custom_id("t01-page", "t01-btn", &serialized, None).unwrap();
    app.store().insert(StoreRecord {
        custom_id: custom_id.0.clone(), definition_kind: DefinitionKind::Page,
        definition_id: "t01-page".into(), execution_id: "t01-btn".into(),
        serialized_data: serialized, frame_identity: None, pagination_state: None, schema_version: 1,
    }).await.unwrap();
    // GREEN partial: no panic; full green requires execution registry population at register().
    let result = app.dispatch_interaction(component_interaction(&custom_id.0)).await;
    assert!(result.is_err(), "without registered process-local callback, dispatch must Err cleanly");
}

#[tokio::test]
async fn t01_modal_submit_route_resolves_store_or_returns_clean_error() {
    let app = app();
    let _modal = Modal::<TestData>::new("t01-modal").set_title("Form")
        .add_text_input("msg", "Message", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _sub: ModalSubmission| async move { Ok(()) }).register(&app).unwrap();
    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = crate::app::store::derive_custom_id("t01-modal", "submit", &serialized, None).unwrap();
    app.store().insert(StoreRecord {
        custom_id: custom_id.0.clone(), definition_kind: DefinitionKind::Modal,
        definition_id: "t01-modal".into(), execution_id: "submit".into(),
        serialized_data: serialized, frame_identity: None, pagination_state: None, schema_version: 1,
    }).await.unwrap();
    let result = app.dispatch_interaction(modal_submit_interaction(&custom_id.0)).await;
    assert!(result.is_err(), "without registered process-local callback, dispatch must Err cleanly");
}

#[tokio::test]
async fn t02_ctx_reply_returns_error_not_panic() {
    let app = app();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.reply("hello").await.is_err());
}

#[tokio::test]
async fn t02_ctx_defer_returns_error_not_panic() {
    let app = app();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.defer().await.is_err());
}

#[tokio::test]
async fn t02_ctx_edit_message_returns_error_not_panic() {
    let app = app();
    let page = Page::<TestData>::new("t02-edit").add_text("hi").register(&app).unwrap();
    let message = page.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.edit_message(message).await.is_err());
}

#[tokio::test]
async fn t02_ctx_edit_original_response_returns_error_not_panic() {
    let app = app();
    let page = Page::<TestData>::new("t02-edit-orig").add_text("hi").register(&app).unwrap();
    let message = page.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.edit_original_response(message).await.is_err());
}

#[tokio::test]
async fn t02_ctx_followup_returns_error_not_panic() {
    let app = app();
    let page = Page::<TestData>::new("t02-followup").add_text("hi").register(&app).unwrap();
    let message = page.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.followup(message).await.is_err());
}

#[tokio::test]
async fn t02_ctx_delete_response_returns_error_not_panic() {
    let app = app();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.delete_response().await.is_err());
}

#[tokio::test]
async fn t02_ctx_open_modal_returns_error_not_panic() {
    let app = app();
    let modal = Modal::<TestData>::new("t02-modal").set_title("Open me")
        .add_text_input("f", "Field", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) }).register(&app).unwrap();
    let rendered = modal.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    assert!(ctx.open_modal(rendered).await.is_err());
}

#[tokio::test]
async fn t03_callback_not_invoked_without_dispatch() {
    let app = app();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let _page = Page::<TestData>::new("t03-page")
        .add_button(Button::new("Click", "t03-btn", ButtonStyle::Secondary, move |_ctx, _data: TestData| {
            let flag = flag.clone();
            async move { flag.store(true, Ordering::SeqCst); Ok(()) }
        })).register(&app).unwrap();
    assert!(!called.load(Ordering::SeqCst));
}

#[test]
fn t04_database_exposes_only_framework_store_not_app_repositories() {
    let app = app();
    let _ = app.store();
    let _ = app.db.store.as_ref();
    assert!(false, "Page API requires application repositories on App/Database");
}

#[tokio::test]
async fn t05_only_memory_store_exists_and_is_not_durable() {
    let store = MemoryStore::default();
    store.insert(StoreRecord {
        custom_id: "durability-probe".into(), definition_kind: DefinitionKind::Page,
        definition_id: "p".into(), execution_id: "e".into(), serialized_data: b"{}".to_vec(),
        frame_identity: None, pagination_state: None, schema_version: 1,
    }).await.unwrap();
    assert!(store.resolve(&CustomId("durability-probe".into())).await.unwrap().is_some());
    assert!(MemoryStore::default().resolve(&CustomId("durability-probe".into())).await.unwrap().is_none());
    assert!(false, "Page API requires restart-surviving custom-ID persistence");
}

#[tokio::test]
async fn t06_modal_render_rejects_string_select_input() {
    let app = app();
    let modal = Modal::<TestData>::new("t06-select").set_title("Select form")
        .add_select("choice", "Pick one", vec![ModalOption::new("A", "a"), ModalOption::new("B", "b")])
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) }).register(&app).unwrap();
    assert!(modal.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn t06_modal_render_rejects_checkbox_input() {
    let app = app();
    let modal = Modal::<TestData>::new("t06-check").set_title("Check form").add_checkbox("agree", "I agree")
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) }).register(&app).unwrap();
    assert!(modal.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn t06_failed_modal_render_must_not_leave_orphan_store_record() {
    let app = app();
    let modal = Modal::<TestData>::new("t06-orphan").set_title("Orphan probe").add_file_upload("upload", "File")
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) }).register(&app).unwrap();
    assert!(modal.render(&app, TestData::default()).await.is_err());
    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = crate::app::store::derive_custom_id("t06-orphan", "submit", &serialized, None).unwrap();
    assert!(app.store().resolve(&custom_id).await.unwrap().is_none());
}

#[test]
fn t07_typed_modal_deserialize_rejects_users() {
    let mut sub = ModalSubmission::default();
    sub.values.insert("who".into(), SubmittedValue::Users(vec![]));
    #[derive(Deserialize)] struct Form { who: String }
    assert!(sub.deserialize::<Form>().is_err());
}

#[test]
fn t07_typed_modal_deserialize_rejects_channels() {
    let mut sub = ModalSubmission::default();
    sub.values.insert("ch".into(), SubmittedValue::Channels(vec![]));
    #[derive(Deserialize)] struct Form { ch: String }
    assert!(sub.deserialize::<Form>().is_err());
}

#[test]
fn t07_typed_modal_deserialize_rejects_files() {
    let mut sub = ModalSubmission::default();
    sub.values.insert("f".into(), SubmittedValue::Files(vec![]));
    #[derive(Deserialize)] struct Form { f: String }
    assert!(sub.deserialize::<Form>().is_err());
}

#[tokio::test]
async fn t08_paginated_frame_lacks_navigation_contract() {
    let app = app();
    let page = Page::<TestData>::new("t08-nav").add_frame(
        Frame::<TestData>::new("big-list", |_ctx, _data| async move {
            Ok((0..30).map(|i| Element::text(format!("Item {i}"))).collect())
        }).paginate(),
    ).register(&app).unwrap();
    let _ = page.render(&app, TestData::default()).await.unwrap();
    assert!(false, "Page API requires framework-managed previous/next navigation");
}

#[tokio::test]
async fn t08_pagination_state_not_persisted() {
    let app = app();
    let page = Page::<TestData>::new("t08-state").add_frame(
        Frame::<TestData>::new("list", |_ctx, _data| async move {
            Ok((0..20).map(|i| Element::text(format!("Row {i}"))).collect())
        }).paginate(),
    ).register(&app).unwrap();
    let _ = page.render(&app, TestData::default()).await.unwrap();
    assert!(false, "paginated render must persist PaginationState in StoreRecord");
}

#[tokio::test]
async fn t09_paginated_frame_inside_container_must_not_over_limit_error() {
    let app = app();
    let page = Page::<TestData>::new("t09-nested").add_container(
        Container::new().add_text("Header").add_frame(
            Frame::<TestData>::new("nested", |_ctx, _data| async move {
                Ok((0..15).map(|i| Element::text(format!("Child {i}"))).collect())
            }).paginate(),
        ),
    ).register(&app).unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_ok(), "paginated Frame inside Container must split; err={}",
        result.as_ref().err().map(|e| e.to_string()).unwrap_or_default());
}

#[tokio::test]
async fn t10_dynamic_text_must_count_toward_pagination_budget() {
    let app = app();
    let page = Page::<TestData>::new("t10-dyn").add_frame(
        Frame::<TestData>::new("dyn", |_ctx, data| {
            let label = data.label.clone();
            async move {
                Ok((0..20).map(|i| Element::text(format!("{label} line {i} {}", "x".repeat(200)))).collect())
            }
        }).paginate(),
    ).register(&app).unwrap();
    match page.render(&app, TestData::default()).await {
        Ok(msg) => {
            let (_c, _a, pagination) = msg.into_parts();
            assert!(pagination.is_some(), "large dynamic Frame should produce pagination state");
        }
        Err(err) => {
            let text = err.to_string();
            assert!(text.contains("limit") || text.contains("budget") || text.contains("character") || text.contains("component"));
        }
    }
}

#[tokio::test]
async fn t11_section_with_button_accessory_in_frame_requires_stable_key() {
    let app = app();
    let page = Page::<TestData>::new("t11-section").add_frame(Frame::<TestData>::new("f", |_ctx, _data| async move {
        Ok(vec![Element::section(
            Section::new().set_text("Info", None, None).set_accessory(
                Button::new("Act", "act-btn", ButtonStyle::Primary, |_ctx, _data: TestData| async move { Ok(()) }),
            ),
        )])
    })).register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn t11_container_with_button_in_frame_requires_stable_key() {
    let app = app();
    let page = Page::<TestData>::new("t11-container").add_frame(Frame::<TestData>::new("f", |_ctx, _data| async move {
        Ok(vec![Element::container(
            Container::new().add_button(Button::new("Nested", "nested-btn", ButtonStyle::Secondary, |_ctx, _data: TestData| async move { Ok(()) })),
        )])
    })).register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[test]
fn t12_direct_closure_syntax_is_required_by_page_api_examples() {
    assert!(false, "Page API requires add_text(|ctx, data| async move {{ ... }}) without dynamic(...)");
}

#[tokio::test]
async fn t13_disabled_select_option_must_render_or_be_rejected_at_build() {
    let app = app();
    let page = Page::<TestData>::new("t13-disabled").add_selection_menu(StringSelect::new(
        "Choose", "t13-sel",
        [SelectOption::new("On", "on"), SelectOption::new("Off", "off").set_disabled(true)],
        |_ctx, _data: TestData, _sel: Vec<String>| async move { Ok(()) },
    )).register(&app).unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_ok(), "disabled SelectOption should render or setter removed; err={}",
        result.as_ref().err().map(|e| e.to_string()).unwrap_or_default());
}

#[tokio::test]
async fn t14_modal_text_input_max_chars_limit_is_enforced() {
    let app = app();
    let modal = Modal::<TestData>::new("t14-len2").set_title("Limits").add_input(ModalInput::Text(TextInput {
        id: "body".into(), label: Value::Static("Body".into()), style: TextInputStyle::Paragraph,
        required: true, placeholder: None, min_length: None, max_length: Some(5000),
    })).on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) });
    let registered = modal.register(&app).unwrap();
    assert!(registered.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn t14_modal_select_max_options_limit_is_enforced() {
    let app = app();
    let options: Vec<_> = (0..30).map(|i| ModalOption::new(format!("Opt {i}"), format!("v{i}"))).collect();
    let modal = Modal::<TestData>::new("t14-opts").set_title("Too many").add_select("pick", "Pick", options)
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) }).register(&app).unwrap();
    let result = modal.render(&app, TestData::default()).await;
    assert!(result.is_err());
    let err = match result { Err(e) => e.to_string(), Ok(_) => String::new() };
    if !err.contains("25") && !err.contains("option") {
        assert!(false, "Modal select over 25 options must fail for the limit: {err}");
    }
}

#[test]
fn t15_page_and_modal_ids_must_be_unique_across_registries() {
    let app = app();
    Page::<TestData>::new("shared-id").add_text("page").register(&app).unwrap();
    let result = Modal::<TestData>::new("shared-id").set_title("Modal")
        .add_text_input("f", "F", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) }).register(&app);
    assert!(result.is_err(), "Page and Modal IDs must be unique across both registries");
}

#[tokio::test]
async fn into_components_keeps_attachments_via_into_parts() {
    let app = app();
    let page = Page::<TestData>::new("attach-page").add_text("see file").add_file(File::new("report.pdf")).register(&app).unwrap();
    let mut rendered = page.render(&app, TestData::default()).await.unwrap();
    rendered = rendered.add_attachment("report.pdf", b"%PDF-demo".to_vec());
    assert!(!rendered.attachments().is_empty());
    let (_components, attachments, _pagination) = rendered.into_parts();
    assert!(!_components.is_empty());
    assert!(!attachments.is_empty());
}

#[tokio::test]
async fn entity_select_menus_must_be_renderable() {
    let app = app();
    let page = Page::<TestData>::new("entity-sel").add_selection_menu(
        UserSelect::new("user-sel", |_ctx, _data: TestData, _users| async move { Ok(()) }).set_placeholder("Pick user"),
    ).register(&app).unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_ok(), "UserSelect must render; err={}", result.as_ref().err().map(|e| e.to_string()).unwrap_or_default());
}

#[test]
fn database_name_collision_with_framework_store_is_a_spec_gap() {
    assert!(false, "G-01: separate host application database from framework Store");
}
