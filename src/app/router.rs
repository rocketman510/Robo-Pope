//! Route interactions by stored identity, never by visual component position.
//!
//! Page API §9: resolve the custom ID via Store, restore serialized data,
//! locate the registered execution, and invoke its callback with the original
//! data and any selection/submission payload.

use std::sync::Arc;

use serde_json::Value as JsonValue;
use serenity::all::{
    CommandDataResolved, ComponentInteractionDataKind, Interaction, InteractionType, LabelComponent,
    ModalComponent, SelectMenuKind,
};

use crate::app::executions::{ExecutionKey, InteractionPayload};
use crate::app::store::CustomId;
use crate::app::{App, Ctx};
use crate::page::{ModalSubmission, Selection};
use crate::page::frame::pagination::{PaginationRequest, PaginationState};
fn routed_execution_id(frame_identity: Option<&str>, execution_id: &str) -> String {
    let identity = frame_identity.unwrap_or_default();
    let frame_id = identity.rsplit_once(":page:")
        .and_then(|(frame, index)| index.parse::<usize>().ok().map(|_| frame))
        .unwrap_or(identity);
    match frame_identity {
        Some(_) => format!("{frame_id}::{execution_id}"),
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
            .resolve_record(&custom_id)
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

        let selection = extract_selection(&component.data.kind, &component.data.resolved)?;
        let payload = if record.execution_id.starts_with("__frame:") {
            let state_value = record.pagination_state.clone()
                .ok_or_else(|| anyhow::anyhow!("Pagination record is missing its state"))?;
            let state: PaginationState = serde_json::from_value(state_value.clone())?;
            let identity = record.frame_identity.as_deref()
                .ok_or_else(|| anyhow::anyhow!("Pagination record is missing its Frame identity"))?;
            let (frame_id, source_page_index) = match identity.rsplit_once(":page:") {
                Some((frame_id, index)) => (frame_id.to_owned(), index.parse::<usize>()?),
                None => (identity.to_owned(), 0),
            };
            anyhow::ensure!(state.frame_id == frame_id, "Pagination record Frame identity does not match its state");
            anyhow::ensure!(state.page_index == source_page_index, "This pagination control is stale");
            let prefix = format!("__frame:{frame_id}:");
            let direction = record.execution_id.strip_prefix(&prefix)
                .ok_or_else(|| anyhow::anyhow!("Malformed framework pagination execution ID"))?;
            let target_page_index = match direction {
                "previous" if source_page_index > 0 => source_page_index - 1,
                "next" if source_page_index + 1 < state.page_count => source_page_index + 1,
                "previous" | "next" => anyhow::bail!("Pagination control is outside the Frame's page range"),
                _ => anyhow::bail!("Unknown framework pagination direction '{direction}'"),
            };
            InteractionPayload::Pagination(PaginationRequest {
                custom_id: custom_id.clone(),
                expected_state: state_value,
                frame_id,
                source_page_index,
                target_page_index,
                page_count: state.page_count,
            })
        } else {
            match &selection {
                Some(s) => InteractionPayload::Selection(s.clone()),
                None => InteractionPayload::None,
            }
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
            .resolve_record(&custom_id)
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

        let submission = extract_modal_submission(modal)?;
        let payload = InteractionPayload::Modal(submission);
        let ctx = Ctx::new(app.clone(), Arc::new(interaction), None);
        callback(ctx, data, payload).await
    }
}


fn parse_u64(value: &str, field: &str) -> anyhow::Result<u64> {
    value.parse::<u64>()
        .map_err(|error| anyhow::anyhow!("Invalid Discord ID in Modal field '{field}': {error}"))
}

/// Reconstructs every submitted Modal value from Serenity's Label components
/// and resolved entity/attachment maps.
fn extract_modal_submission(
    modal: &serenity::all::ModalInteraction,
) -> anyhow::Result<ModalSubmission> {
    let mut submission = ModalSubmission::default();
    for component in &modal.data.components {
        let ModalComponent::Label(label) = component else {
            continue;
        };
        match &label.component {
            LabelComponent::InputText(input) => {
                submission.values.insert(
                    input.custom_id.to_string(),
                    crate::page::SubmittedValue::Text(input.value.to_string()),
                );
            }
            LabelComponent::SelectMenu(menu) => {
                let id = menu.custom_id.to_string();
                let values: Vec<String> = menu.values.iter().map(ToString::to_string).collect();
                let value = match &menu.kind {
                    SelectMenuKind::String { .. } => crate::page::SubmittedValue::Strings(values),
                    SelectMenuKind::User { .. } => {
                        let selected = values.iter().map(|value| {
                            Ok(crate::page::SelectedUser {
                                id: serenity::all::UserId::new(parse_u64(value, &id)?),
                                resolved: None,
                            })
                        }).collect::<anyhow::Result<Vec<_>>>()?;
                        crate::page::SubmittedValue::Users(selected)
                    }
                    SelectMenuKind::Role { .. } => {
                        let selected = values.iter().map(|value| {
                            Ok(crate::page::SelectedRole {
                                id: serenity::all::RoleId::new(parse_u64(value, &id)?),
                                resolved: None,
                            })
                        }).collect::<anyhow::Result<Vec<_>>>()?;
                        crate::page::SubmittedValue::Roles(selected)
                    }
                    SelectMenuKind::Mentionable { .. } => {
                        let mut selected = Vec::with_capacity(values.len());
                        for value in &values {
                            let raw_id = parse_u64(value, &id)?;
                            let user_id = serenity::all::UserId::new(raw_id);
                            let role_id = serenity::all::RoleId::new(raw_id);
                            if modal.data.resolved.users.get(&user_id).is_some() {
                                selected.push(crate::page::SelectedMentionable::User(
                                    crate::page::SelectedUser { id: user_id, resolved: None }
                                ));
                            } else if modal.data.resolved.roles.get(&role_id).is_some() {
                                selected.push(crate::page::SelectedMentionable::Role(
                                    crate::page::SelectedRole { id: role_id, resolved: None }
                                ));
                            } else {
                                anyhow::bail!(
                                    "Modal field '{id}' contains a Mentionable ID absent from resolved entities"
                                );
                            }
                        }
                        crate::page::SubmittedValue::Mentionables(selected)
                    }
                    SelectMenuKind::Channel { .. } => {
                        let selected = values.iter().map(|value| {
                            Ok(crate::page::SelectedChannel {
                                id: serenity::all::ChannelId::new(parse_u64(value, &id)?),
                                resolved: None,
                            })
                        }).collect::<anyhow::Result<Vec<_>>>()?;
                        crate::page::SubmittedValue::Channels(selected)
                    }
                    _ => anyhow::bail!("Unsupported Modal Select kind in field '{id}'"),
                };
                submission.values.insert(id, value);
            }
            LabelComponent::FileUpload(upload) => {
                let id = upload.custom_id.to_string();
                let mut attachments = Vec::with_capacity(upload.values.len() as usize);
                for attachment_id in &upload.values {
                    let attachment = modal.data.resolved.attachments.get(attachment_id)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!(
                            "Modal field '{id}' references an attachment missing from resolved data"
                        ))?;
                    attachments.push(attachment);
                }
                submission.values.insert(id, crate::page::SubmittedValue::Files(attachments));
            }
            LabelComponent::RadioGroup(group) => {
                let value = group.value.as_ref().map(|value| vec![value.to_string()]).unwrap_or_default();
                submission.values.insert(
                    group.custom_id.to_string(),
                    crate::page::SubmittedValue::Strings(value),
                );
            }
            LabelComponent::CheckboxGroup(group) => {
                submission.values.insert(
                    group.custom_id.to_string(),
                    crate::page::SubmittedValue::Strings(group.values.iter().map(ToString::to_string).collect()),
                );
            }
            LabelComponent::Checkbox(checkbox) => {
                submission.values.insert(
                    checkbox.custom_id.to_string(),
                    crate::page::SubmittedValue::Boolean(checkbox.value),
                );
            }
            _ => anyhow::bail!("Unsupported or future Modal Label component type"),
        }
    }
    Ok(submission)
}

fn extract_selection(
    kind: &ComponentInteractionDataKind,
    resolved: &CommandDataResolved,
) -> anyhow::Result<Option<Selection>> {
    Ok(match kind {
        ComponentInteractionDataKind::StringSelect { values } => {
            Some(Selection::Strings(values.iter().map(|value| value.to_string()).collect()))
        }
        ComponentInteractionDataKind::UserSelect { values } => Some(Selection::Users(
            values.iter().map(|id| crate::page::SelectedUser {
                id: *id,
                resolved: resolved.users.get(id).cloned().map(Arc::new),
            }).collect(),
        )),
        ComponentInteractionDataKind::RoleSelect { values } => Some(Selection::Roles(
            values.iter().map(|id| crate::page::SelectedRole {
                id: *id,
                resolved: resolved.roles.get(id).cloned().map(Arc::new),
            }).collect(),
        )),
        ComponentInteractionDataKind::MentionableSelect { values } => {
            let mut selected = Vec::with_capacity(values.len() as usize);
            for generic_id in values {
                let user_id = serenity::all::UserId::new(generic_id.get());
                let role_id = serenity::all::RoleId::new(generic_id.get());
                if let Some(user) = resolved.users.get(&user_id) {
                    selected.push(crate::page::SelectedMentionable::User(
                        crate::page::SelectedUser {
                            id: user_id,
                            resolved: Some(Arc::new(user.clone())),
                        }
                    ));
                } else if let Some(role) = resolved.roles.get(&role_id) {
                    selected.push(crate::page::SelectedMentionable::Role(
                        crate::page::SelectedRole {
                            id: role_id,
                            resolved: Some(Arc::new(role.clone())),
                        }
                    ));
                } else {
                    anyhow::bail!(
                        "Mentionable Select returned ID '{}' without a resolved User or Role",
                        generic_id
                    );
                }
            }
            Some(Selection::Mentionables(selected))
        }
        ComponentInteractionDataKind::ChannelSelect { values } => Some(Selection::Channels(
            values.iter().map(|id| crate::page::SelectedChannel {
                id: serenity::all::ChannelId::new(id.get()),
                resolved: None,
            }).collect(),
        )),
        ComponentInteractionDataKind::Button => None,
        _ => None,
    })
}
