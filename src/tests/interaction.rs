use std::sync::Arc;

use serde_json::json;
use serenity::all::Interaction;

use crate::app::{App, Ctx};
use super::support::app;

fn ping_interaction() -> Interaction {
    serde_json::from_str(r#"{"
type": 1,
        "id": "100",
        "application_id": "200",
        "token": "test-interaction-token",
        "version": 1,
        "app_permissions": "0"
    }"#)
    .expect("minimal Ping interaction fixture must deserialize")
}

#[test]
fn context_construction_retains_app_http_and_absent_ping_metadata() {
    let app = app();
    let interaction = Arc::new(ping_interaction());

    let ctx = Ctx::new(app.clone(), interaction, None);

    assert!(Arc::ptr_eq(&ctx.app.db, &app.db));
    assert!(Arc::ptr_eq(&ctx.http, &app.http));
    assert!(ctx.user_id().is_none());
    assert!(ctx.guild_id().is_none());
    assert!(ctx.channel_id().is_none());
    assert!(ctx.selected().is_none());
}

#[tokio::test]
async fn router_returns_an_error_for_an_unsupported_interaction_kind() {
    let app: App = app();

    let result = app.dispatch_interaction(ping_interaction()).await;

    assert!(result.is_err());
}
#[test]
fn context_retains_the_typed_selection_payload() {
    use crate::page::Selection;

    let app = app();
    let selection = Selection::Strings(vec!["member".into(), "admin".into()]);
    let ctx = Ctx::new(app, Arc::new(ping_interaction()), Some(selection));

    assert!(matches!(
        ctx.selected(),
        Some(Selection::Strings(values)) if values == &["member".to_string(), "admin".to_string()]
    ));
}
