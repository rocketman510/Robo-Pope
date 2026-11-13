use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use crate::app::{App, Ctx, Database};
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

async fn mock_discord(max_requests: usize) -> (String, tokio::task::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap().to_string();
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for index in 0..max_requests {
            let accepted = timeout(Duration::from_millis(750), listener.accept()).await;
            let Ok(Ok((mut stream, _))) = accepted else { break; };
            requests.push(read_http_request(&mut stream).await);
            let (status, body) = if index == 8 {
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

#[tokio::test]
async fn every_context_response_method_uses_its_correct_discord_http_operation() {
    // FIRST: each response method sends its distinct Discord operation; an
    // interaction may only receive one initial response.
    let (address, server) = mock_discord(9).await;
    let http = Arc::new(
        serenity::http::HttpBuilder::without_token()
            .application_id(serenity::all::ApplicationId::new(200))
            .proxy(format!("http://{address}"))
            .ratelimiter_disabled(true)
            .build()
    );
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    let app = App::new(http, Database::new(()), store);
    let interaction = || Arc::new(component_interaction("response-test"));

    Ctx::new(app.clone(), interaction(), None).reply("hello").await.unwrap();
    Ctx::new(app.clone(), interaction(), None).defer().await.unwrap();

    let page = Page::<TestData>::new("response-edit").add_text("edited").register(&app).unwrap();
    Ctx::new(app.clone(), interaction(), None).edit_message(page.render(&app, TestData::default()).await.unwrap()).await.unwrap();
    Ctx::new(app.clone(), interaction(), None).edit_original_response(page.render(&app, TestData::default()).await.unwrap()).await.unwrap();
    Ctx::new(app.clone(), interaction(), None).followup(page.render(&app, TestData::default()).await.unwrap()).await.unwrap();
    Ctx::new(app.clone(), interaction(), None).delete_response().await.unwrap();

    let modal = Modal::<TestData>::new("response-modal").set_title("Open")
        .add_text_input("field", "Field", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    let modal_render = modal.render(&app, TestData::default()).await.unwrap();
    Ctx::new(app.clone(), interaction(), None).open_modal(modal_render).await.unwrap();

    let once_ctx = Ctx::new(app, interaction(), None);
    once_ctx.reply("first response").await.unwrap();
    assert!(once_ctx.defer().await.is_err());

    let requests = server.await.unwrap();
    assert!(requests.len() >= 8);
    let lines: Vec<_> = requests.iter().map(|request| request.lines().next().unwrap_or("")).collect();
    assert!(lines[0].contains("POST /api/v10/interactions/9001/component-token/callback"));
    assert!(lines[1].contains("POST /api/v10/interactions/9001/component-token/callback"));
    assert!(lines[2].contains("POST /api/v10/interactions/9001/component-token/callback"));
    assert!(lines[3].contains("PATCH /api/v10/webhooks/200/component-token/messages/@original"));
    assert!(lines[4].contains("POST /api/v10/webhooks/200/component-token"));
    assert!(lines[5].contains("DELETE /api/v10/webhooks/200/component-token/messages/@original"));
    assert!(lines[6].contains("POST /api/v10/interactions/9001/component-token/callback"));

    let body = |index: usize| requests[index].split_once("\r\n\r\n").unwrap().1;
    let reply: serde_json::Value = serde_json::from_str(body(0)).unwrap();
    let defer: serde_json::Value = serde_json::from_str(body(1)).unwrap();
    let edit: serde_json::Value = serde_json::from_str(body(2)).unwrap();
    let open_modal: serde_json::Value = serde_json::from_str(body(6)).unwrap();
    assert_eq!(reply["type"], 4);
    assert!(defer["type"] == 5 || defer["type"] == 6);
    assert_eq!(edit["type"], 7);
    assert_eq!(open_modal["type"], 9);
}
