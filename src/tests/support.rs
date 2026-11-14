//! Shared test fixtures for isolated framework contract tests.
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::app::App;
use crate::app::store::{MemoryStore, Store};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TestData { pub label: String, pub count: u32 }
impl Default for TestData {
    fn default() -> Self { Self { label: "sample".into(), count: 7 } }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OtherData { pub enabled: bool }

pub fn app() -> App<Arc<()>> {
    let store: Arc<dyn Store> = Arc::new(MemoryStore::default());
    App::new(Arc::new(serenity::http::HttpBuilder::without_token().build()), Arc::new(()), store)
}
pub fn app_with_http_and_store(http: Arc<serenity::all::Http>, store: Arc<dyn Store>) -> App<Arc<()>> {
    App::new(http, Arc::new(()), store)
}
pub fn ping_interaction() -> serenity::all::Interaction {
    serde_json::from_str(r#"{"type":1,"id":"100","application_id":"200","token":"test-token","version":1,"app_permissions":"0"}"#)
        .expect("Ping fixture must deserialize")
}
pub fn component_interaction(custom_id: &str) -> serenity::all::Interaction {
    let payload = format!(r#"{{
        "type":3,"id":"9001","application_id":"200","token":"component-token",
        "version":1,"app_permissions":"0","locale":"en-US","guild_locale":null,
        "entitlements":[],"attachment_size_limit":10485760,
        "channel":{{"id":"300","type":0,"name":"test-channel"}},
        "channel_id":"300",
        "user":{{"id":"400","username":"tester","discriminator":"0","global_name":null,"avatar":null}},
        "message":{{
            "id":"500","channel_id":"300",
            "author":{{"id":"200","username":"bot","discriminator":"0","global_name":null,"avatar":null}},
            "content":"","timestamp":"2024-01-01T00:00:00.000000+00:00","edited_timestamp":null,
            "tts":false,"mention_everyone":false,"mentions":[],"mention_roles":[],
            "attachments":[],"embeds":[],"pinned":false,"type":0,"flags":0,"components":[]
        }},
        "data":{{"custom_id":"{custom_id}","component_type":2}}
    }}"#);
    serde_json::from_str(&payload).expect("Component interaction fixture must deserialize")
}
pub fn modal_submit_interaction(custom_id: &str) -> serenity::all::Interaction {
    let payload = format!(r#"{{
        "type":5,"id":"9002","application_id":"200","token":"modal-token",
        "version":1,"app_permissions":"0","locale":"en-US","guild_locale":null,
        "entitlements":[],"attachment_size_limit":10485760,
        "channel":{{"id":"300","type":0,"name":"test-channel"}},"channel_id":"300",
        "user":{{"id":"400","username":"tester","discriminator":"0","global_name":null,"avatar":null}},
        "data":{{"custom_id":"{custom_id}","components":[]}}
    }}"#);
    serde_json::from_str(&payload).expect("Modal submit fixture must deserialize")
}
