//! Gateway adapter: App owns framework state; AppHandler receives gateway events.
use serenity::{all::{Context, EventHandler, FullEvent}, async_trait};
use crate::app::App;

/// Serenity gateway adapter that forwards gateway events to the App.
pub struct AppHandler<D = crate::app::Database> {
    /// The single App instance that owns framework state.
    pub app: App<D>,
}
impl<D> AppHandler<D>
where
    D: Clone + std::any::Any + Send + Sync + 'static,
{
    /// Creates an event handler bound to the supplied App.
    pub fn new(app: App<D>) -> Self { Self { app } }

    /// Routes one gateway interaction through the App's interaction router.
    pub async fn dispatch_interaction(&self, interaction: serenity::all::Interaction) -> anyhow::Result<()> {
        self.app.dispatch_interaction(interaction).await
    }
}
#[async_trait]
impl<D> EventHandler for AppHandler<D>
where
    D: Clone + std::any::Any + Send + Sync + 'static,
{
    async fn dispatch(&self, _ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready { data_about_bot, .. } => {
                println!("{} is connected. Demo definitions are registered.", data_about_bot.user.name);
                println!("Demo Page registered: {}", self.app.pages.contains("framework-demo"));
                println!("Demo Modal registered: {}", self.app.modals.contains("framework-demo-modal"));
            }
            FullEvent::InteractionCreate { interaction, .. } => {
                // Page API §§8–9: App owns the router and receives incoming interactions.
                if let Err(err) = self.dispatch_interaction(interaction.clone()).await {
                    eprintln!("Interaction routing error: {err:#}");
                }
            }
            _ => {}
        }
    }
}
