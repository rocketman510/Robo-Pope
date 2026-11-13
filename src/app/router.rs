//! Route interactions by stored identity, never by visual component position.
use crate::app::App;
use serenity::all::Interaction;

#[derive(Clone, Default)]
pub struct InteractionRouter;
impl InteractionRouter {
    pub async fn dispatch(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        todo!("identify interaction kind and delegate to its route")
    }
    pub async fn dispatch_component(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        todo!("resolve Store record, definition, execution, Page data, and selection payload")
    }
    pub async fn dispatch_modal_submit(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        todo!("resolve Modal data and pass ModalSubmission separately")
    }
}
