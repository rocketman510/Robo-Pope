//! Gateway adapter: App owns framework state; AppHandler receives gateway events.
use serenity::{all::{Context, EventHandler, FullEvent}, async_trait};
use crate::app::App;

pub struct AppHandler { pub app: App }
impl AppHandler {
    pub fn new(app: App) -> Self { Self { app } }
}
#[async_trait]
impl EventHandler for AppHandler {
    async fn dispatch(&self, _ctx: &Context, event: &FullEvent) {
        if let FullEvent::InteractionCreate { interaction, .. } = event {
            todo!("dispatch InteractionCreate through App and log contextual anyhow errors")
        }
    }
}
