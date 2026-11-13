//! Route interactions by stored identity, never by visual component position.
use crate::app::App;
use serenity::all::Interaction;

#[derive(Clone, Default)]
/// Resolves Discord interaction IDs to the Page or Modal execution they name.
///
/// Routing restores the original serialized data and selection/submission
/// payload. The resolved ID is not an authorization decision.
pub struct InteractionRouter;
impl InteractionRouter {
    /// Dispatches a supported interaction kind to its specialized route.
    pub async fn dispatch(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        todo!("identify interaction kind and delegate to its route")
    }
    /// Resolves and invokes a registered component execution from its custom ID.
    pub async fn dispatch_component(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        todo!("resolve Store record, definition, execution, Page data, and selection payload")
    }
    /// Resolves the Modal execution and supplies submitted values separately
    /// from the Modal's originally serialized data.
    pub async fn dispatch_modal_submit(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        todo!("resolve Modal data and pass ModalSubmission separately")
    }
}
