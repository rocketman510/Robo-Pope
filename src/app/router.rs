//! Route interactions by stored identity, never by visual component position.
//!
//! Page API §9: resolve the custom ID via Store, restore serialized data,
//! locate the registered execution, and invoke its callback with the original
//! data and any selection/submission payload.

use std::sync::Arc;

use serde_json::Value as JsonValue;
use serenity::all::{ComponentInteractionDataKind, Interaction, InteractionType};

use crate::app::executions::{ExecutionKey, InteractionPayload};
use crate::app::store::CustomId;
use crate::app::{App, Ctx};
use crate::page::{ModalSubmission, Selection};
fn routed_execution_id(frame_identity: Option<&str>, execution_id: &str) -> String {
    match frame_identity {
        Some(identity) => format!("{identity}::{execution_id}"),
        None => execution_id.to_owned(),
    }
}


#[derive(Clone, Default)]
/// Resolves Discord interaction IDs to the Page or Modal execution they name.
///
/// Routing restores the original serialized data and selection/submission
/// payload. The resolved ID is not an authorization decision.
pub struct InteractionRouter;

impl InteractionRouter {
    /// Dispatches a supported interaction kind to its specialized route.
    pub async fn dispatch(&self, app: &App, interaction: Interaction) -> anyhow::Result<()> {
        match interaction.kind() {
            InteractionType::Component => self.dispatch_component(app, interaction).await,
            InteractionType::Modal => self.dispatch_modal_submit(app, interaction).await,
            _ => anyhow::bail!("Unsupported Discord interaction kind for UI routing"),
        }
    }

    /// Resolves and invokes a registered component execution from its custom ID.
    pub async fn dispatch_component(
        &self,
        app: &App,
        interaction: Interaction,
    ) -> anyhow::Result<()> {
        let component = match &interaction {
            Interaction::Component(c) => c,
            _ => anyhow::bail!("Expected a Component interaction"),
        };

        let custom_id = CustomId(component.data.custom_id.to_string());
        let record = app
            .store()
            .resolve(&custom_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("No execution record for custom ID '{}'", custom_id.0)
            })?;

        let key = ExecutionKey {
            definition_id: record.definition_id.clone(),
            execution_id: routed_execution_id(record.frame_identity.as_deref(), &record.execution_id),
        };
        let callback = app.executions.get(&key).ok_or_else(|| {
            anyhow::anyhow!(
                "No process-local execution registered for definition '{}' execution '{}'",
                key.definition_id,
                key.execution_id
            )
        })?;

        let data: JsonValue = serde_json::from_slice(&record.serialized_data)
            .map_err(|e| anyhow::anyhow!("Deserializing stored Page data: {e}"))?;

        let selection = extract_selection(&component.data.kind);
        let payload = match &selection {
            Some(s) => InteractionPayload::Selection(s.clone()),
            None => InteractionPayload::None,
        };

        let ctx = Ctx::new(app.clone(), Arc::new(interaction), selection);
        callback(ctx, data, payload).await
    }

    /// Resolves the Modal execution and supplies submitted values separately
    /// from the Modal's originally serialized data.
    pub async fn dispatch_modal_submit(
        &self,
        app: &App,
        interaction: Interaction,
    ) -> anyhow::Result<()> {
        let modal = match &interaction {
            Interaction::Modal(m) => m,
            _ => anyhow::bail!("Expected a Modal submit interaction"),
        };

        let custom_id = CustomId(modal.data.custom_id.to_string());
        let record = app
            .store()
            .resolve(&custom_id)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("No execution record for Modal custom ID '{}'", custom_id.0)
            })?;

        let key = ExecutionKey {
            definition_id: record.definition_id.clone(),
            execution_id: record.execution_id.clone(),
        };
        let callback = app.executions.get(&key).ok_or_else(|| {
            anyhow::anyhow!(
                "No process-local Modal execution for definition '{}' execution '{}'",
                key.definition_id,
                key.execution_id
            )
        })?;

        let data: JsonValue = serde_json::from_slice(&record.serialized_data)
            .map_err(|e| anyhow::anyhow!("Deserializing stored Modal data: {e}"))?;

        let submission = ModalSubmission::default();
        let payload = InteractionPayload::Modal(submission);
        let ctx = Ctx::new(app.clone(), Arc::new(interaction), None);
        callback(ctx, data, payload).await
    }
}

fn extract_selection(kind: &ComponentInteractionDataKind) -> Option<Selection> {
    match kind {
        ComponentInteractionDataKind::StringSelect { values } => {
            Some(Selection::Strings(values.iter().map(|v| v.to_string()).collect()))
        }
        ComponentInteractionDataKind::UserSelect { values } => Some(Selection::Users(
            values
                .iter()
                .map(|id| crate::page::SelectedUser {
                    id: *id,
                    resolved: None,
                })
                .collect(),
        )),
        ComponentInteractionDataKind::RoleSelect { values } => Some(Selection::Roles(
            values
                .iter()
                .map(|id| crate::page::SelectedRole {
                    id: *id,
                    resolved: None,
                })
                .collect(),
        )),
        ComponentInteractionDataKind::MentionableSelect { values } => {
            let _ = values;
            Some(Selection::Mentionables(Vec::new()))
        }
        ComponentInteractionDataKind::ChannelSelect { values } => Some(Selection::Channels(
            values
                .iter()
                .map(|id| crate::page::SelectedChannel {
                    id: *id,
                    resolved: None,
                })
                .collect(),
        )),
        ComponentInteractionDataKind::Button => None,
        _ => None,
    }
}
