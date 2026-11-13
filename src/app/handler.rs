//! Gateway adapter: App owns framework state; AppHandler receives gateway events.
use serenity::{all::{Context, EventHandler, FullEvent}, async_trait};
use crate::app::App;

/// Serenity gateway adapter that forwards interaction events to the App.
pub struct AppHandler {
    /// The single App instance that owns framework state.
    pub app: App,
}
impl AppHandler {
    /// Creates an event handler bound to the supplied App.
    pub fn new(app: App) -> Self { Self { app } }
}
#[async_trait]
impl EventHandler for AppHandler {
    /// Forwards interaction-creation events; unrelated gateway events are ignored.
    async fn dispatch(&self, _ctx: &Context, event: &FullEvent) {
        if let FullEvent::InteractionCreate { interaction, .. } = event {
            todo!("dispatch InteractionCreate through App and log contextual anyhow errors")
        }
    }
}
