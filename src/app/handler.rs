//! Gateway adapter: App owns framework state; AppHandler receives gateway events.
use serenity::{all::{Context, EventHandler, FullEvent}, async_trait};
use crate::app::App;

/// Serenity gateway adapter that forwards gateway events to the App.
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
    /// Reports a successful Gateway connection. Interaction routing is not
    /// invoked here yet because the router and Page renderer are still pending.
    async fn dispatch(&self, _ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready { data_about_bot, .. } => {
                println!("{} is connected. Demo definitions are registered.", data_about_bot.user.name);
                println!("Demo Page registered: {}", self.app.pages.contains("framework-demo"));
                println!("Demo Modal registered: {}", self.app.modals.contains("framework-demo-modal"));
            }
            FullEvent::InteractionCreate { .. } => {
                println!("Received an interaction. Framework interaction routing is not implemented yet.");
            }
            _ => {}
        }
    }
}
