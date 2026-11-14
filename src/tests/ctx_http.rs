use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use crate::app::{App, Ctx};
use crate::app::store::{MemoryStore, Store};
use crate::page::{Modal, ModalSubmission, Page, TextInputStyle};
use super::support::{component_interaction, TestData};

const MESSAGE_JSON: &str = r#"{"id":"500","channel_id":"300","author":{"id":"200","username":"bot","discriminator":"0","global_name":null,"avatar":null},"content":"","timestamp":"2024-01-01T00:00:00.000000+00:00","edited_timestamp":null,"tts":false,"mention_everyone":false,"mentions":[],"mention_roles":[],"attachments":[],"embeds":[],"pinned":false,"type":0,"flags":0,"components":[]}"#;

async fn read_http_request(stream: &mut TcpStream) -> String {
    let mut bytes = Vec::new();
    let mut expected_len = None;
    let mut header_end = None;
    loop {
        let mut buffer = [0_u8; 4096];
        let read = stream.read(&mut buffer).await.unwrap();
        if read == 0 { break; }
        bytes.extend_from_slice(&buffer[..read]);
        if header_end.is_none() {
            if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                let end = index + 4;
                let headers = String::from_utf8_lossy(&bytes[..end]);
                let length = headers.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok()).flatten()
                }).unwrap_or(0);
                header_end = Some(end);
                expected_len = Some(length);
            }
        }
        if let (Some(end), Some(length)) = (header_end, expected_len) {
            if bytes.len() >= end + length { break; }
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

async fn mock_discord(max_requests: usize, reject_index: Option<usize>)
    -> (String, tokio::task::JoinHandle<Vec<String>>)
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for index in 0..max_requests {
            let accepted = timeout(Duration::from_millis(750), listener.accept()).await;
            let Ok(Ok((mut stream, _))) = accepted else { break; };
            requests.push(read_http_request(&mut stream).await);
            let (status, body) = if reject_index == Some(index) {
                ("400 Bad Request", r#"{"message":"Interaction has already been acknowledged","code":40060}"#)
            } else { ("200 OK", MESSAGE_JSON) };
            let response = format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
            stream.shutdown().await.unwrap();
        }
        requests
    });
    (address, task)
}

async fn setup(max_requests: usize, reject_index: Option<usize>)
    -> (App<Arc<()>>, tokio::task::JoinHandle<Vec<String>>)
{
    let (address, server) = mock_discord(max_requests, reject_index).await;
    let http = Arc::new(
        serenity::http::HttpBuilder::without_token()
            .application_id(serenity::all::ApplicationId::new(200))
            .proxy(format!("http://{address}"))
            .ratelimiter_disabled(true)
            .build()
    );
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    (App::new(http, Arc::new(()), store), server)
}

fn context(app: &App<Arc<()>>) -> Ctx {
    Ctx::new(app.clone(), Arc::new(component_interaction("response-test")), None)
}

fn assert_callback_type(request: &str, response_type: u64) {
    assert!(request.lines().next().unwrap().contains(
        "POST /api/v10/interactions/9001/component-token/callback"
    ));
    let body = request.split_once("\r\n\r\n").unwrap().1;
    let payload: serde_json::Value = serde_json::from_str(body).unwrap();
    assert_eq!(payload["type"], response_type);
}

#[tokio::test]
async fn reply_sends_a_channel_message_interaction_response() {
    // FIRST: reply maps to Discord's ChannelMessageWithSource callback.
    let (app, server) = setup(1, None).await;
    context(&app).reply("hello").await.unwrap();
    let requests = server.await.unwrap();
    assert_callback_type(&requests[0], 4);
}

#[tokio::test]
async fn defer_sends_a_deferred_interaction_response() {
    // FIRST: defer acknowledges the interaction with the appropriate defer response.
    let (app, server) = setup(1, None).await;
    context(&app).defer().await.unwrap();
    let requests = server.await.unwrap();
    assert!(requests[0].lines().next().unwrap().contains("/interactions/9001/component-token/callback"));
    let body = requests[0].split_once("\r\n\r\n").unwrap().1;
    let payload: serde_json::Value = serde_json::from_str(body).unwrap();
    assert!(payload["type"] == 5 || payload["type"] == 6);
}

#[tokio::test]
async fn edit_message_sends_an_update_message_interaction_response() {
    // FIRST: editing the component's message is the UpdateMessage callback, not a webhook edit.
    let (app, server) = setup(1, None).await;
    let page = Page::<TestData>::new("ctx-edit-message").add_text("updated").register(&app).unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    context(&app).edit_message(rendered).await.unwrap();
    let requests = server.await.unwrap();
    assert_callback_type(&requests[0], 7);
}

#[tokio::test]
async fn edit_original_response_uses_the_original_webhook_message_route() {
    // FIRST: editing the original webhook response is distinct from UpdateMessage.
    let (app, server) = setup(1, None).await;
    let page = Page::<TestData>::new("ctx-edit-original").add_text("updated").register(&app).unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    context(&app).edit_original_response(rendered).await.unwrap();
    let requests = server.await.unwrap();
    assert!(requests[0].lines().next().unwrap().contains(
        "PATCH /api/v10/webhooks/200/component-token/messages/@original"
    ));
}

#[tokio::test]
async fn followup_creates_a_separate_webhook_message() {
    // FIRST: followup creates a new webhook message rather than editing the original.
    let (app, server) = setup(1, None).await;
    let page = Page::<TestData>::new("ctx-followup").add_text("follow-up").register(&app).unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    context(&app).followup(rendered).await.unwrap();
    let requests = server.await.unwrap();
    assert!(requests[0].lines().next().unwrap().contains("POST /api/v10/webhooks/200/component-token"));
}

#[tokio::test]
async fn delete_response_deletes_the_original_webhook_message() {
    // FIRST: deleting the response uses Discord's original-webhook DELETE route.
    let (app, server) = setup(1, None).await;
    context(&app).delete_response().await.unwrap();
    let requests = server.await.unwrap();
    assert!(requests[0].lines().next().unwrap().contains(
        "DELETE /api/v10/webhooks/200/component-token/messages/@original"
    ));
}

#[tokio::test]
async fn open_modal_sends_a_modal_interaction_response() {
    // FIRST: Modal opening is the Modal callback response type.
    let (app, server) = setup(1, None).await;
    let modal = Modal::<TestData>::new("ctx-open-modal").set_title("Open")
        .add_text_input("field", "Field", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    let rendered = modal.render(&app, TestData::default()).await.unwrap();
    context(&app).open_modal(rendered).await.unwrap();
    let requests = server.await.unwrap();
    assert_callback_type(&requests[0], 9);
}

#[tokio::test]
async fn an_interaction_cannot_receive_two_initial_responses() {
    // FIRST: once acknowledged, a second initial response is rejected.
    let (app, server) = setup(2, Some(1)).await;
    let ctx = context(&app);
    ctx.reply("first").await.unwrap();
    assert!(ctx.defer().await.is_err());
    let requests = server.await.unwrap();
    assert!(requests.len() == 1 || requests.len() == 2);
}
