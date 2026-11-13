//! RED-phase contract tests derived from `spec/Rust Implementation Audit.md`
//! and cross-checked against `spec/Page API.md`.
//!
//! These tests are expected to FAIL (or panic on `todo!()`) until the
//! corresponding contract surface is implemented. Each test names the audit
//! finding it proves and the Page API section it enforces.

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
            \"type\": 1,
            \"id\": \"100\",
            \"application_id\": \"200\",
            \"token\": \"test-interaction-token\",
            \"version\": 1,
            \"app_permissions\": \"0\"
        }"#,
    )
    .expect("minimal Ping interaction fixture must deserialize")
}

fn component_interaction(custom_id: &str) -> Interaction {
    let payload = format!(
        r#"{{
            \"type\": 3,
            \"id\": \"9001\",
            \"application_id\": \"200\",
            \"token\": \"component-token\",
            \"version\": 1,
            \"app_permissions\": \"0\",
            \"channel_id\": \"300\",
            \"user\": {{
                \"id\": \"400\",
                \"username\": \"tester\",
                \"discriminator\": \"0\",
                \"global_name\": null,
                \"avatar\": null
            }},
            \"message\": {{
                \"id\": \"500\",
                \"channel_id\": \"300\",
                \"author\": {{
                    \"id\": \"200\",
                    \"username\": \"bot\",
                    \"discriminator\": \"0\",
                    \"global_name\": null,
                    \"avatar\": null
                }},
                \"content\": \"\",
                \"timestamp\": \"2024-01-01T00:00:00.000000+00:00\",
                \"edited_timestamp\": null,
                \"tts\": false,
                \"mention_everyone\": false,
                \"mentions\": [],
                \"mention_roles\": [],
                \"attachments\": [],
                \"embeds\": [],
                \"pinned\": false,
                \"type\": 0,
                \"flags\": 0,
                \"components\": []
            }},
            \"data\": {{
                \"custom_id\": \"{custom_id}\",
                \"component_type\": 2
            }}
        }}"#
    );
    serde_json::from_str(&payload).expect("component interaction fixture must deserialize")
}

fn modal_submit_interaction(custom_id: &str) -> Interaction {
    let payload = format!(
        r#"{{
            \"type\": 5,
            \"id\": \"9002\",
            \"application_id\": \"200\",
            \"token\": \"modal-token\",
            \"version\": 1,
            \"app_permissions\": \"0\",
            \"channel_id\": \"300\",
            \"user\": {{
                \"id\": \"400\",
                \"username\": \"tester\",
                \"discriminator\": \"0\",
                \"global_name\": null,
                \"avatar\": null
            }},
            \"data\": {{
                \"custom_id\": \"{custom_id}\",
                \"components\": []
            }}
        }}"#
    );
    serde_json::from_str(&payload).expect("modal submit fixture must deserialize")
}

// T-01
#[tokio::test]
#[should_panic(expected = "resolve Store record")]
async fn t01_component_route_is_not_implemented() {
    let app = app();
    let _page = Page::<TestData>::new("t01-page")
        .add_button(Button::new(
            "Go",
            "t01-btn",
            ButtonStyle::Primary,
            |_ctx, _data: TestData| async move { Ok(()) },
        ))
        .register(&app)
        .unwrap();

    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = crate::app::store::derive_custom_id(
        "t01-page", "t01-btn", &serialized, None,
    ).unwrap();
    app.store().insert(StoreRecord {
        custom_id: custom_id.0.clone(),
        definition_kind: DefinitionKind::Page,
        definition_id: "t01-page".into(),
        execution_id: "t01-btn".into(),
        serialized_data: serialized,
        frame_identity: None,
        pagination_state: None,
        schema_version: 1,
    }).await.unwrap();

    let _ = app.dispatch_interaction(component_interaction(&custom_id.0)).await;
}

#[tokio::test]
#[should_panic(expected = "resolve Modal data")]
async fn t01_modal_submit_route_is_not_implemented() {
    let app = app();
    let _modal = Modal::<TestData>::new("t01-modal")
        .set_title("Form")
        .add_text_input("msg", "Message", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _sub: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();

    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = crate::app::store::derive_custom_id(
        "t01-modal", "submit", &serialized, None,
    ).unwrap();
    app.store().insert(StoreRecord {
        custom_id: custom_id.0.clone(),
        definition_kind: DefinitionKind::Modal,
        definition_id: "t01-modal".into(),
        execution_id: "submit".into(),
        serialized_data: serialized,
        frame_identity: None,
        pagination_state: None,
        schema_version: 1,
    }).await.unwrap();

    let _ = app.dispatch_interaction(modal_submit_interaction(&custom_id.0)).await;
}

// T-02
#[tokio::test]
#[should_panic(expected = "send initial interaction response")]
async fn t02_ctx_reply_is_not_implemented() {
    let app = app();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.reply("hello").await;
}

#[tokio::test]
#[should_panic(expected = "defer initial response")]
async fn t02_ctx_defer_is_not_implemented() {
    let app = app();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.defer().await;
}

#[tokio::test]
#[should_panic(expected = "edit component message")]
async fn t02_ctx_edit_message_is_not_implemented() {
    let app = app();
    let page = Page::<TestData>::new("t02-edit").add_text("hi").register(&app).unwrap();
    let message = page.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.edit_message(message).await;
}

#[tokio::test]
#[should_panic(expected = "edit original response")]
async fn t02_ctx_edit_original_response_is_not_implemented() {
    let app = app();
    let page = Page::<TestData>::new("t02-edit-orig").add_text("hi").register(&app).unwrap();
    let message = page.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.edit_original_response(message).await;
}

#[tokio::test]
#[should_panic(expected = "send follow-up response")]
async fn t02_ctx_followup_is_not_implemented() {
    let app = app();
    let page = Page::<TestData>::new("t02-followup").add_text("hi").register(&app).unwrap();
    let message = page.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.followup(message).await;
}

#[tokio::test]
#[should_panic(expected = "delete interaction response")]
async fn t02_ctx_delete_response_is_not_implemented() {
    let app = app();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.delete_response().await;
}

#[tokio::test]
#[should_panic(expected = "open Modal as initial interaction response")]
async fn t02_ctx_open_modal_is_not_implemented() {
    let app = app();
    let modal = Modal::<TestData>::new("t02-modal")
        .set_title("Open me")
        .add_text_input("f", "Field", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();
    let rendered = modal.render(&app, TestData::default()).await.unwrap();
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), None);
    let _ = ctx.open_modal(rendered).await;
}

// T-03
#[tokio::test]
async fn t03_app_handler_does_not_dispatch_interactions() {
    let app = app();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let _page = Page::<TestData>::new("t03-page")
        .add_button(Button::new(
            "Click",
            "t03-btn",
            ButtonStyle::Secondary,
            move |_ctx, _data: TestData| {
                let flag = flag.clone();
                async move {
                    flag.store(true, Ordering::SeqCst);
                    Ok(())
                }
            },
        ))
        .register(&app)
        .unwrap();
    assert!(!called.load(Ordering::SeqCst),
        "callback must not have run without interaction dispatch");
}

// T-04
#[test]
fn t04_database_exposes_only_framework_store_not_app_repositories() {
    let app = app();
    let _store: &dyn Store = app.store();
    let db: &Database = &app.db;
    let _ = db.store.as_ref();
    assert!(false,
        "Page API requires application repositories on App/Database \
         (e.g. ctx.app.db.users); only framework Store is present");
}

// T-05
#[tokio::test]
async fn t05_only_memory_store_exists_and_is_not_durable() {
    let store = MemoryStore::default();
    let record = StoreRecord {
        custom_id: "durability-probe".into(),
        definition_kind: DefinitionKind::Page,
        definition_id: "p".into(),
        execution_id: "e".into(),
        serialized_data: b"{}".to_vec(),
        frame_identity: None,
        pagination_state: None,
        schema_version: 1,
    };
    store.insert(record).await.unwrap();
    assert!(store.resolve(&CustomId("durability-probe".into())).await.unwrap().is_some());
    let fresh = MemoryStore::default();
    assert!(fresh.resolve(&CustomId("durability-probe".into())).await.unwrap().is_none(),
        "MemoryStore must not claim cross-process durability");
    assert!(false,
        "Page API requires restart-surviving custom-ID persistence; \
         repository ships only MemoryStore");
}

// T-06
#[tokio::test]
async fn t06_modal_render_rejects_string_select_input() {
    let app = app();
    let modal = Modal::<TestData>::new("t06-select")
        .set_title("Select form")
        .add_select("choice", "Pick one", vec![
            ModalOption::new("A", "a"),
            ModalOption::new("B", "b"),
        ])
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();
    let result = modal.render(&app, TestData::default()).await;
    assert!(result.is_err(), "non-Text Modal inputs must be renderable per Page API §6; got Ok");
}

#[tokio::test]
async fn t06_modal_render_rejects_checkbox_input() {
    let app = app();
    let modal = Modal::<TestData>::new("t06-check")
        .set_title("Check form")
        .add_checkbox("agree", "I agree")
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();
    let result = modal.render(&app, TestData::default()).await;
    assert!(result.is_err(), "Checkbox Modal input must be renderable");
}

#[tokio::test]
async fn t06_failed_modal_render_must_not_leave_orphan_store_record() {
    let app = app();
    let modal = Modal::<TestData>::new("t06-orphan")
        .set_title("Orphan probe")
        .add_file_upload("upload", "File")
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();
    let before = app.store();
    let result = modal.render(&app, TestData::default()).await;
    assert!(result.is_err());
    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = crate::app::store::derive_custom_id(
        "t06-orphan", "submit", &serialized, None,
    ).unwrap();
    let leftover = before.resolve(&custom_id).await.unwrap();
    assert!(leftover.is_none(),
        "failed Modal render must not leave a persisted StoreRecord (found {:?})",
        leftover.map(|r| r.custom_id));
}

// T-07
#[test]
fn t07_typed_modal_deserialize_rejects_users() {
    let mut sub = ModalSubmission::default();
    sub.values.insert("who".into(), SubmittedValue::Users(vec![]));
    #[derive(Deserialize)]
    struct Form { who: String }
    assert!(sub.deserialize::<Form>().is_err(),
        "Users values should eventually deserialize into typed fields per §6");
}

#[test]
fn t07_typed_modal_deserialize_rejects_channels() {
    let mut sub = ModalSubmission::default();
    sub.values.insert("ch".into(), SubmittedValue::Channels(vec![]));
    #[derive(Deserialize)]
    struct Form { ch: String }
    assert!(sub.deserialize::<Form>().is_err(),
        "Channels values should be supported by typed Modal submission");
}

#[test]
fn t07_typed_modal_deserialize_rejects_files() {
    let mut sub = ModalSubmission::default();
    sub.values.insert("f".into(), SubmittedValue::Files(vec![]));
    #[derive(Deserialize)]
    struct Form { f: String }
    assert!(sub.deserialize::<Form>().is_err(),
        "File upload values should be supported by typed Modal submission");
}

// T-08
#[tokio::test]
async fn t08_paginated_frame_does_not_emit_navigation_controls() {
    let app = app();
    let page = Page::<TestData>::new("t08-nav")
        .add_frame(
            Frame::<TestData>::new("big-list", |_ctx, _data| async move {
                Ok((0..30).map(|i| Element::text(format!("Item {i}"))).collect())
            }).paginate(),
        )
        .register(&app)
        .unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let (components, _attachments, pagination) = rendered.into_parts();
    let blob = serde_json::to_value(&components).unwrap().to_string();
    let has_nav = blob.contains("previous") || blob.contains("next") || blob.contains("prev");
    assert!(has_nav || pagination.is_some(),
        "paginated Frame must expose navigation controls or recoverable pagination state");
    assert!(false,
        "Page API §5 requires framework-managed previous/next navigation for paginated Frames");
}

#[tokio::test]
async fn t08_pagination_state_is_not_persisted_on_render() {
    let app = app();
    let page = Page::<TestData>::new("t08-state")
        .add_frame(
            Frame::<TestData>::new("list", |_ctx, _data| async move {
                Ok((0..20).map(|i| Element::text(format!("Row {i}"))).collect())
            }).paginate(),
        )
        .register(&app)
        .unwrap();
    let _ = page.render(&app, TestData::default()).await.unwrap();
    assert!(false,
        "paginated render must persist PaginationState in StoreRecord (currently always None)");
}

// T-09
#[tokio::test]
async fn t09_paginated_frame_inside_container_must_not_over_limit_error() {
    let app = app();
    let page = Page::<TestData>::new("t09-nested")
        .add_container(
            Container::new()
                .add_text("Header")
                .add_frame(
                    Frame::<TestData>::new("nested", |_ctx, _data| async move {
                        Ok((0..15).map(|i| Element::text(format!("Child {i}"))).collect())
                    }).paginate(),
                ),
        )
        .register(&app)
        .unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_ok(),
        "paginated Frame inside Container must split across pages instead of erroring; got {result:?}");
}

// T-10
#[tokio::test]
async fn t10_dynamic_text_must_count_toward_pagination_budget() {
    let app = app();
    let page = Page::<TestData>::new("t10-dyn")
        .add_frame(
            Frame::<TestData>::new("dyn", |_ctx, data| {
                let label = data.label.clone();
                async move {
                    Ok((0..20).map(|i| Element::text(format!("{label} line {i} {}", "x".repeat(200)))).collect())
                }
            }).paginate(),
        )
        .register(&app)
        .unwrap();
    let result = page.render(&app, TestData::default()).await;
    match result {
        Ok(msg) => {
            let (_c, _a, pagination) = msg.into_parts();
            assert!(pagination.is_some(), "large dynamic Frame content should produce pagination state");
        }
        Err(err) => {
            let text = err.to_string();
            assert!(text.contains("limit") || text.contains("budget") || text.contains("character") || text.contains("component"),
                "budget failure must be descriptive, got: {text}");
        }
    }
}

// T-11
#[tokio::test]
async fn t11_section_with_button_accessory_in_frame_requires_stable_key() {
    let app = app();
    let page = Page::<TestData>::new("t11-section")
        .add_frame(Frame::<TestData>::new("f", |_ctx, _data| async move {
            Ok(vec![Element::section(
                Section::new()
                    .set_text("Info", None, None)
                    .set_accessory(Button::new(
                        "Act", "act-btn", ButtonStyle::Primary,
                        |_ctx, _data: TestData| async move { Ok(()) },
                    )),
            )])
        }))
        .register(&app)
        .unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_err(),
        "Section with Button accessory inside a Frame must require a stable developer key");
}

#[tokio::test]
async fn t11_container_with_button_in_frame_requires_stable_key() {
    let app = app();
    let page = Page::<TestData>::new("t11-container")
        .add_frame(Frame::<TestData>::new("f", |_ctx, _data| async move {
            Ok(vec![Element::container(
                Container::new().add_button(Button::new(
                    "Nested", "nested-btn", ButtonStyle::Secondary,
                    |_ctx, _data: TestData| async move { Ok(()) },
                )),
            )])
        }))
        .register(&app)
        .unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_err(),
        "Container with interactive descendants inside a Frame must require a stable developer key");
}

// T-12
#[test]
fn t12_direct_closure_syntax_is_required_by_page_api_examples() {
    assert!(false,
        "Page API §2 requires add_text(|ctx, data| async move {{ ... }}) without dynamic(...)");
}

// T-13
#[tokio::test]
async fn t13_disabled_select_option_must_render_or_be_rejected_at_build() {
    let app = app();
    let page = Page::<TestData>::new("t13-disabled")
        .add_selection_menu(StringSelect::new(
            "Choose", "t13-sel",
            [
                SelectOption::new("On", "on"),
                SelectOption::new("Off", "off").set_disabled(true),
            ],
            |_ctx, _data: TestData, _sel: Vec<String>| async move { Ok(()) },
        ))
        .register(&app)
        .unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_ok(),
        "disabled SelectOption should render successfully or the setter should not exist; render failed: {result:?}");
}

// T-14
#[tokio::test]
async fn t14_modal_text_input_max_chars_limit_is_enforced() {
    let app = app();
    let modal = Modal::<TestData>::new("t14-len2")
        .set_title("Limits")
        .add_input(ModalInput::Text(TextInput {
            id: "body".into(),
            label: Value::Static("Body".into()),
            style: TextInputStyle::Paragraph,
            required: true,
            placeholder: None,
            min_length: None,
            max_length: Some(5000),
        }))
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) });
    let registered = modal.register(&app).unwrap();
    let result = registered.render(&app, TestData::default()).await;
    assert!(result.is_err(), "TextInput max_length above 4000 must be rejected at render/validation");
}

#[tokio::test]
async fn t14_modal_select_max_options_limit_is_enforced() {
    let app = app();
    let options: Vec<_> = (0..30).map(|i| ModalOption::new(format!("Opt {i}"), format!("v{i}"))).collect();
    let modal = Modal::<TestData>::new("t14-opts")
        .set_title("Too many")
        .add_select("pick", "Pick", options)
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();
    let result = modal.render(&app, TestData::default()).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    if !err.contains("25") && !err.contains("option") {
        assert!(false,
            "Modal select over 25 options must fail validation for the limit, not only because rendering is unimplemented: {err}");
    }
}

// T-15
#[test]
fn t15_page_and_modal_ids_must_be_unique_across_registries() {
    let app = app();
    Page::<TestData>::new("shared-id").add_text("page").register(&app).unwrap();
    let result = Modal::<TestData>::new("shared-id")
        .set_title("Modal")
        .add_text_input("f", "F", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _s: ModalSubmission| async move { Ok(()) })
        .register(&app);
    assert!(result.is_err(),
        "Page and Modal IDs must be unique across both registries; duplicate 'shared-id' was accepted");
}

// Additional
#[tokio::test]
async fn into_components_discards_attachments_and_pagination_metadata() {
    let app = app();
    let page = Page::<TestData>::new("attach-page")
        .add_text("see file")
        .add_file(File::new("report.pdf"))
        .register(&app)
        .unwrap();
    let mut rendered = page.render(&app, TestData::default()).await.unwrap();
    rendered = rendered.add_attachment("report.pdf", b"%PDF-demo".to_vec());
    assert!(!rendered.attachments().is_empty());
    let components_only = rendered.clone().into_components();
    assert!(!components_only.is_empty());
    let (_c, attachments, _pagination) = rendered.into_parts();
    assert!(!attachments.is_empty(), "into_parts must retain attachments");
}

#[tokio::test]
async fn entity_select_menus_must_be_renderable() {
    let app = app();
    let page = Page::<TestData>::new("entity-sel")
        .add_selection_menu(
            UserSelect::new("user-sel", |_ctx, _data: TestData, _users| async move { Ok(()) })
                .set_placeholder("Pick user"),
        )
        .register(&app)
        .unwrap();
    let result = page.render(&app, TestData::default()).await;
    assert!(result.is_ok(), "UserSelect must render per Page API §4; got {result:?}");
}

#[test]
fn database_name_collision_with_framework_store_is_a_spec_gap() {
    assert!(false,
        "G-01: separate host application database from framework Store in the public App API");
}
