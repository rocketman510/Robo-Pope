//! Public-API integration tests for complete Page, Store, and interaction flows.
//!
//! These tests intentionally live under tests/ rather than src/tests/. They
//! compile as an external consumer of robo_pope and cannot access crate-private
//! constructors or implementation details. Discord HTTP is replaced with a
//! local mock server; no bot token or live Discord connection is required.

use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};

use robo_pope::{
    app::{
        App, Ctx, RenderCtx,
        store::{
            CustomId, MemoryStore, RoutingRecord, SqliteStore, Store, StoreDurability,
            derive_custom_id,
        },
    },
    page::{
        Button, ButtonStyle, Container, Element, LinkButton, MediaGallery, Modal,
        Page, Section, SelectOption, Separator, StringSelect, TextInputStyle, Thumbnail,
    },
};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{ApplicationId, Interaction},
    http::HttpBuilder,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
    time::timeout,
};

const MESSAGE_JSON: &str = r#"{"id":"500","channel_id":"300","author":{"id":"200","username":"bot","discriminator":"0","global_name":null,"avatar":null},"content":"","timestamp":"2024-01-01T00:00:00.000000+00:00","edited_timestamp":null,"tts":false,"mention_everyone":false,"mentions":[],"mention_roles":[],"attachments":[],"embeds":[],"pinned":false,"type":0,"flags":0,"components":[]}"#;

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Default)]
struct DemoDatabase {
    button_clicks: Arc<AtomicUsize>,
    selections: Arc<Mutex<Vec<String>>>,
    submitted_message: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DemoData {
    username: String,
    visits: u64,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct DemoForm {
    message: String,
}

fn app_with_store(store: Arc<dyn Store>) -> App<DemoDatabase> {
    let http = Arc::new(HttpBuilder::without_token().build());
    App::new(http, DemoDatabase::default(), store)
}

fn memory_app() -> App<DemoDatabase> {
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    app_with_store(store)
}

fn demo_data() -> DemoData {
    DemoData {
        username: "Integration Tester".into(),
        visits: 12,
    }
}

fn serialized_data(data: &DemoData) -> Vec<u8> {
    serde_json::to_vec(data).expect("demo data should serialize")
}

fn component_interaction(custom_id: &str, component_type: u64, values: &[&str]) -> Interaction {
    let mut data = serde_json::json!({
        "custom_id": custom_id,
        "component_type": component_type,
    });
    if component_type == 3 {
        data["values"] = serde_json::json!(values);
    }

    let payload = serde_json::json!({
        "type": 3,
        "id": "9001",
        "application_id": "200",
        "token": "component-token",
        "version": 1,
        "app_permissions": "0",
        "locale": "en-US",
        "guild_locale": null,
        "entitlements": [],
        "attachment_size_limit": 10485760,
        "channel": {"id": "300", "type": 0, "name": "test-channel"},
        "channel_id": "300",
        "user": {
            "id": "400",
            "username": "tester",
            "discriminator": "0",
            "global_name": null,
            "avatar": null
        },
        "message": {
            "id": "500",
            "channel_id": "300",
            "author": {
                "id": "200",
                "username": "bot",
                "discriminator": "0",
                "global_name": null,
                "avatar": null
            },
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
        },
        "data": data
    });

    serde_json::from_value(payload).expect("Component interaction fixture should deserialize")
}

fn button_interaction(custom_id: &str) -> Interaction {
    component_interaction(custom_id, 2, &[])
}

fn string_select_interaction(custom_id: &str, values: &[&str]) -> Interaction {
    component_interaction(custom_id, 3, values)
}

fn modal_submit_interaction(custom_id: &str, message: &str) -> Interaction {
    let payload = serde_json::json!({
        "type": 5,
        "id": "9002",
        "application_id": "200",
        "token": "modal-token",
        "version": 1,
        "app_permissions": "0",
        "locale": "en-US",
        "guild_locale": null,
        "entitlements": [],
        "attachment_size_limit": 10485760,
        "channel": {"id": "300", "type": 0, "name": "test-channel"},
        "channel_id": "300",
        "user": {
            "id": "400",
            "username": "tester",
            "discriminator": "0",
            "global_name": null,
            "avatar": null
        },
        "data": {
            "custom_id": custom_id,
            "components": [{
                "type": 18,
                "id": 1,
                "label": "Your message",
                "component": {
                    "type": 4,
                    "id": 2,
                    "custom_id": "message",
                    "value": message,
                    "style": 1
                }
            }]
        }
    });

    serde_json::from_value(payload).expect("Modal submission fixture should deserialize")
}

async fn read_http_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut expected_len = None;
    let mut header_end = None;

    loop {
        let mut buffer = [0_u8; 4096];
        let read = stream.read(&mut buffer).await.expect("read mock request");
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);

        if header_end.is_none() {
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                let end = index + 4;
                let headers = String::from_utf8_lossy(&bytes[..end]);
                let length = headers.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                }).unwrap_or(0);
                header_end = Some(end);
                expected_len = Some(length);
            }
        }

        if let (Some(end), Some(length)) = (header_end, expected_len) {
            if bytes.len() >= end + length {
                break;
            }
        }
    }

    String::from_utf8_lossy(&bytes).into_owned()
}

async fn mock_discord() -> (String, JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind local Discord mock");
    let address = listener.local_addr().expect("mock address").to_string();

    let server = tokio::spawn(async move {
        let (mut stream, _) = timeout(Duration::from_secs(5), listener.accept())
            .await
            .expect("Discord client should contact the local mock")
            .expect("accept local Discord connection");

        let request = read_http_request(&mut stream).await;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            MESSAGE_JSON.len(),
            MESSAGE_JSON
        );
        stream.write_all(response.as_bytes()).await.expect("write mock response");
        stream.shutdown().await.expect("close mock response");
        request
    });

    (address, server)
}

async fn app_with_mock_discord() -> (App<DemoDatabase>, JoinHandle<String>) {
    let (address, server) = mock_discord().await;
    let http = Arc::new(
        HttpBuilder::without_token()
            .application_id(ApplicationId::new(200))
            .proxy(format!("http://{address}"))
            .ratelimiter_disabled(true)
            .build(),
    );
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    (App::new(http, DemoDatabase::default(), store), server)
}

fn request_line(request: &str) -> &str {
    request.lines().next().expect("HTTP request line")
}

fn request_payload(request: &str) -> serde_json::Value {
    let (_, body) = request
        .split_once("\r\n\r\n")
        .expect("HTTP request should have headers and body");
    serde_json::from_str(body).expect("Discord request body should be JSON")
}

fn assert_interaction_callback(request: &str, callback_type: u64) -> serde_json::Value {
    assert!(
        request_line(request).contains("/api/v10/interactions/"),
        "expected an interaction callback request, got: {}",
        request_line(request)
    );
    let payload = request_payload(request);
    assert_eq!(payload["type"], callback_type);
    payload
}

#[tokio::test]
async fn public_page_api_renders_dynamic_values_frames_and_components_and_persists_routes() {
    let app = memory_app();
    let frame_calls = Arc::new(AtomicUsize::new(0));
    let frame_calls_in_callback = frame_calls.clone();

    let page = app
        .page::<DemoData>("integration-showcase")
        .add_text("# Integration showcase")
        .add_text(
            |ctx: RenderCtx<DemoDatabase>, data: Arc<DemoData>| async move {
                let clicks = ctx.app.db.button_clicks.load(Ordering::SeqCst);
                Ok(format!(
                    "Welcome, {}. Visits: {}. Clicks: {}.",
                    data.username, data.visits, clicks
                ))
            },
        )
        .add_section(
            Section::new()
                .set_text("Section component", "Rendered from the public API.", None)
                .set_accessory(
                    Thumbnail::new("https://cdn.discordapp.com/embed/avatars/0.png")
                        .set_description("Test thumbnail"),
                ),
        )
        .add_container(
            Container::new()
                .set_accent_color(0x5865F2)
                .add_text("## Container component")
                .add_separator(Separator::new())
                .add_text("Container children preserve order."),
        )
        .add_button_row((
            Button::new(
                "Refresh",
                "refresh",
                ButtonStyle::Primary,
                None,
                false,
                |_ctx: Ctx<DemoDatabase>, _data: DemoData| async move { Ok(()) },
            ),
            None,
            None,
            None,
            None,
        ))
        .add_link_button(LinkButton::new(
            "Repository",
            "https://github.com/rocketman510/Robo-Pope",
            None,
            false,
        ))
        .add_selection_menu(StringSelect::new(
            "Choose an option",
            "choose",
            [
                SelectOption::new("First", "first"),
                SelectOption::new("Second", "second"),
            ],
            |_ctx: Ctx<DemoDatabase>, _data: DemoData, _selected: Vec<String>| async move {
                Ok(())
            },
        ))
        .add_gallery(
            MediaGallery::new()
                .add_media("https://cdn.discordapp.com/embed/avatars/0.png")
                .add_media("https://cdn.discordapp.com/embed/avatars/1.png"),
        )
        .add_frame(move |ctx: RenderCtx<DemoDatabase>, data: Arc<DemoData>| {
            let calls = frame_calls_in_callback.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                let clicks = ctx.app.db.button_clicks.load(Ordering::SeqCst);
                Ok(vec![Element::text(format!(
                    "Frame for {}. Current clicks: {clicks}.",
                    data.username
                ))])
            }
        })
        .register(&app)
        .expect("Page should register through the public API");

    let data = demo_data();
    let bytes = serialized_data(&data);
    let rendered = page
        .render(&app, data.clone())
        .await
        .expect("Page should render to one message");
    let components = rendered.into_components();
    let json = serde_json::to_value(components).expect("rendered components serialize");
    let text = json.to_string();

    assert!(text.contains("Integration showcase"));
    assert!(text.contains("Welcome, Integration Tester"));
    assert!(text.contains("Section component"));
    assert!(text.contains("Container component"));
    assert!(text.contains("Repository"));
    assert!(text.contains("https://github.com/rocketman510/Robo-Pope"));
    assert!(text.contains("First"));
    assert!(text.contains("Second"));
    assert!(text.contains("Frame for Integration Tester"));
    assert_eq!(frame_calls.load(Ordering::SeqCst), 1, "Frame resolves once per render");

    let button_id = derive_custom_id("integration-showcase", "refresh", &bytes).unwrap();
    let select_id = derive_custom_id("integration-showcase", "choose", &bytes).unwrap();
    let button_record = app
        .store()
        .resolve(&button_id)
        .await
        .unwrap()
        .expect("interactive Button route should be persisted");
    let select_record = app
        .store()
        .resolve(&select_id)
        .await
        .unwrap()
        .expect("String Select route should be persisted");

    assert_eq!(button_record.definition_id, "integration-showcase");
    assert_eq!(button_record.execution_id, "refresh");
    assert_eq!(button_record.serialized_data, bytes);
    assert_eq!(select_record.execution_id, "choose");

    let link_id = derive_custom_id("integration-showcase", "repository-link", &serialized_data(&data)).unwrap();
    assert!(
        app.store().resolve(&link_id).await.unwrap().is_none(),
        "URL LinkButtons must not create callback routes"
    );
}

#[tokio::test]
async fn button_dispatch_restores_page_data_updates_typed_database_and_edits_the_message() {
    let (app, server) = app_with_mock_discord().await;
    let page = app
        .page::<DemoData>("integration-refresh")
        .add_text(|ctx: RenderCtx<DemoDatabase>, _data: Arc<DemoData>| async move {
            Ok(format!(
                "Button clicks: {}",
                ctx.app.db.button_clicks.load(Ordering::SeqCst)
            ))
        })
        .add_button_row((
            Button::new(
                "Refresh",
                "refresh",
                ButtonStyle::Primary,
                None,
                false,
                |ctx: Ctx<DemoDatabase>, data: DemoData| async move {
                    assert_eq!(data, demo_data(), "router should restore the serialized Page data");
                    ctx.app.db.button_clicks.fetch_add(1, Ordering::SeqCst);

                    let updated = ctx
                        .app
                        .pages
                        .query::<DemoData>("integration-refresh")?
                        .render(&ctx.app, data)
                        .await?;
                    ctx.edit_message(updated).await?;
                    Ok(())
                },
            ),
            None,
            None,
            None,
            None,
        ))
        .register(&app)
        .unwrap();

    let data = demo_data();
    let custom_id = derive_custom_id("integration-refresh", "refresh", &serialized_data(&data)).unwrap();
    page.render(&app, data).await.unwrap();

    app.dispatch_interaction(button_interaction(&custom_id.0))
        .await
        .expect("Button interaction should route to its callback");

    let request = server.await.expect("mock Discord request should finish");
    let payload = assert_interaction_callback(&request, 7);
    assert_eq!(app.db.button_clicks.load(Ordering::SeqCst), 1);
    assert!(
        payload["data"]["components"]
            .to_string()
            .contains("Button clicks: 1"),
        "edit_message should contain the Page rerendered after the callback"
    );
}

#[tokio::test]
async fn string_select_dispatch_restores_selected_values_and_replies() {
    let (app, server) = app_with_mock_discord().await;
    let page = app
        .page::<DemoData>("integration-selection")
        .add_selection_menu(StringSelect::new(
            "Choose",
            "choose",
            [
                SelectOption::new("Alpha", "alpha"),
                SelectOption::new("Beta", "beta"),
                SelectOption::new("Gamma", "gamma"),
            ],
            |ctx: Ctx<DemoDatabase>, data: DemoData, selected: Vec<String>| async move {
                assert_eq!(data.username, "Integration Tester");
                *ctx.app.db.selections.lock().unwrap() = selected.clone();
                ctx.reply(format!("Selected: {}", selected.join(", "))).await?;
                Ok(())
            },
        ))
        .register(&app)
        .unwrap();

    let data = demo_data();
    let custom_id = derive_custom_id("integration-selection", "choose", &serialized_data(&data)).unwrap();
    page.render(&app, data).await.unwrap();

    app.dispatch_interaction(string_select_interaction(&custom_id.0, &["beta", "gamma"]))
        .await
        .expect("String Select interaction should route to its callback");

    let request = server.await.expect("mock Discord request should finish");
    let payload = assert_interaction_callback(&request, 4);
    assert_eq!(
        *app.db.selections.lock().unwrap(),
        vec!["beta".to_string(), "gamma".to_string()]
    );
    assert_eq!(payload["data"]["content"], "Selected: beta, gamma");
}

#[tokio::test]
async fn button_callback_queries_renders_and_opens_a_registered_modal() {
    let (app, server) = app_with_mock_discord().await;

    Modal::<DemoData, DemoDatabase>::new("integration-edit-modal")
        .set_title("Edit demo data")
        .add_text_input("message", "Your message", TextInputStyle::Short)
        .on_submit_typed(
            |_ctx: Ctx<DemoDatabase>, _data: DemoData, _form: DemoForm| async move { Ok(()) },
        )
        .register(&app)
        .unwrap();

    let page = app
        .page::<DemoData>("integration-open-modal")
        .add_button_row((
            Button::new(
                "Open form",
                "open-form",
                ButtonStyle::Primary,
                None,
                false,
                |ctx: Ctx<DemoDatabase>, data: DemoData| async move {
                    let modal = ctx
                        .app
                        .modals
                        .query::<DemoData>("integration-edit-modal")?
                        .render(&ctx.app, data)
                        .await?;
                    ctx.open_modal(modal).await?;
                    Ok(())
                },
            ),
            None,
            None,
            None,
            None,
        ))
        .register(&app)
        .unwrap();

    let data = demo_data();
    let custom_id = derive_custom_id("integration-open-modal", "open-form", &serialized_data(&data)).unwrap();
    page.render(&app, data).await.unwrap();

    app.dispatch_interaction(button_interaction(&custom_id.0))
        .await
        .expect("Button should open the registered Modal");

    let request = server.await.expect("mock Discord request should finish");
    let payload = assert_interaction_callback(&request, 9);
    assert_eq!(payload["data"]["title"], "Edit demo data");
    assert_eq!(payload["data"]["components"].as_array().unwrap().len(), 1);

    let modal_id = derive_custom_id(
        "integration-edit-modal",
        "submit",
        &serialized_data(&demo_data()),
    )
    .unwrap();
    assert!(
        app.store().resolve(&modal_id).await.unwrap().is_some(),
        "rendering the Modal must persist its submit route before it is opened"
    );
}

#[tokio::test]
async fn modal_submit_dispatch_deserializes_typed_form_data_and_replies() {
    let (app, server) = app_with_mock_discord().await;

    Modal::<DemoData, DemoDatabase>::new("integration-modal-submit")
        .set_title("Submit message")
        .add_text_input("message", "Your message", TextInputStyle::Short)
        .on_submit_typed(
            |ctx: Ctx<DemoDatabase>, data: DemoData, form: DemoForm| async move {
                assert_eq!(data, demo_data(), "Modal routing should restore its original data");
                *ctx.app.db.submitted_message.lock().unwrap() = Some(form.message.clone());
                ctx.reply(format!("Received: {}", form.message)).await?;
                Ok(())
            },
        )
        .register(&app)
        .unwrap();

    let data = demo_data();
    let custom_id = derive_custom_id(
        "integration-modal-submit",
        "submit",
        &serialized_data(&data),
    )
    .unwrap();

    let modal = app
        .modals
        .query::<DemoData>("integration-modal-submit")
        .unwrap();
    modal.render(&app, data).await.unwrap();

    app.dispatch_interaction(modal_submit_interaction(&custom_id.0, "Hello from the form"))
        .await
        .expect("Modal submission should route and deserialize its input");

    let request = server.await.expect("mock Discord request should finish");
    let payload = assert_interaction_callback(&request, 4);
    assert_eq!(
        *app.db.submitted_message.lock().unwrap(),
        Some("Hello from the form".into())
    );
    assert_eq!(payload["data"]["content"], "Received: Hello from the form");
}


#[tokio::test]
async fn frame_generated_button_is_registered_and_dispatched_by_its_page_execution_id() {
    let app = memory_app();
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_in_callback = calls.clone();

    let page = app
        .page::<DemoData>("integration-frame-button")
        .add_frame(move |_ctx: RenderCtx<DemoDatabase>, _data: Arc<DemoData>| {
            let calls = calls_in_callback.clone();
            async move {
                Ok(vec![
                    Element::text("Dynamically generated controls"),
                    Element::button(Button::new(
                        "Frame action",
                        "frame-action",
                        ButtonStyle::Secondary,
                        None,
                        false,
                        move |_ctx: Ctx<DemoDatabase>, data: DemoData| {
                            let calls = calls.clone();
                            async move {
                                assert_eq!(data, demo_data());
                                calls.fetch_add(1, Ordering::SeqCst);
                                Ok(())
                            }
                        },
                    )),
                ])
            }
        })
        .register(&app)
        .unwrap();

    let data = demo_data();
    let custom_id = derive_custom_id(
        "integration-frame-button",
        "frame-action",
        &serialized_data(&data),
    )
    .unwrap();

    page.render(&app, data).await.unwrap();
    assert!(
        app.store().resolve(&custom_id).await.unwrap().is_some(),
        "Frame-generated Button route should be persisted"
    );

    app.dispatch_interaction(button_interaction(&custom_id.0))
        .await
        .expect("Frame-generated Button should dispatch");

    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invalid_resolved_frame_does_not_persist_any_of_its_interaction_routes() {
    let app = memory_app();
    let page = app
        .page::<DemoData>("integration-invalid-frame")
        .add_frame(|_ctx: RenderCtx<DemoDatabase>, _data: Arc<DemoData>| async move {
            let mut elements = Vec::new();
            for index in 0..21 {
                let id = format!("overflow-{index}");
                elements.push(Element::button(Button::new(
                    format!("Button {index}"),
                    id,
                    ButtonStyle::Primary,
                    None,
                    false,
                    |_ctx: Ctx<DemoDatabase>, _data: DemoData| async move { Ok(()) },
                )));
            }
            Ok(elements)
        })
        .register(&app)
        .unwrap();

    let data = demo_data();
    let bytes = serialized_data(&data);
    assert!(
        page.render(&app, data).await.is_err(),
        "an over-budget resolved Frame must fail instead of being truncated or paginated"
    );

    for index in 0..21 {
        let id = derive_custom_id(
            "integration-invalid-frame",
            &format!("overflow-{index}"),
            &bytes,
        )
        .unwrap();
        assert!(
            app.store().resolve(&id).await.unwrap().is_none(),
            "failed Page rendering must not leave an orphaned route"
        );
    }
}

#[tokio::test]
async fn sqlite_store_preserves_exact_routing_records_across_reopen() {
    let unique = NEXT_TEMP_FILE.fetch_add(1, Ordering::SeqCst);
    let path: PathBuf = std::env::temp_dir().join(format!(
        "robo-pope-integration-{}-{unique}.sqlite",
        std::process::id()
    ));

    let data = serialized_data(&demo_data());
    let custom_id = derive_custom_id("sqlite-page", "sqlite-execution", &data).unwrap();
    let record = RoutingRecord {
        hash: custom_id.0.clone(),
        definition_id: "sqlite-page".into(),
        execution_id: "sqlite-execution".into(),
        serialized_data: data,
    };

    {
        let store = SqliteStore::open(&path).await.unwrap();
        assert_eq!(store.durability(), StoreDurability::Durable);
        store.insert(record.clone()).await.unwrap();
    }

    {
        let store = SqliteStore::open(&path).await.unwrap();
        assert_eq!(
            store.resolve(&CustomId(custom_id.0.clone())).await.unwrap(),
            Some(record.clone()),
            "a reopened SQLite Store must return all four routing fields unchanged"
        );
        store.remove(&custom_id).await.unwrap();
        assert!(store.resolve(&custom_id).await.unwrap().is_none());
    }

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(path.with_extension("sqlite-wal"));
    let _ = std::fs::remove_file(path.with_extension("sqlite-shm"));

    let fields: BTreeSet<String> = serde_json::to_value(record)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert_eq!(
        fields,
        ["definition_id", "execution_id", "hash", "serialized_data"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    );
}
