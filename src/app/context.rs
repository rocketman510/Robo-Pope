//! Temporary contexts supplied to dynamic values and interactive executions.
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use serenity::all::{Http, Interaction};
use serenity::builder::{
    CreateAttachment, CreateInteractionResponse, CreateInteractionResponseFollowup,
    CreateInteractionResponseMessage, CreateModal, EditAttachments, EditInteractionResponse,
};
use crate::app::{App, BoxFuture};
use crate::page::{ModalRender, RenderedMessage, Selection};

/// Render-time context for resolving dynamic Page properties and Frame contents.
pub struct RenderCtx<D = Arc<()>> {
    pub app: App<D>,
    pub definition_id: Arc<str>,
}
impl<D: Clone> Clone for RenderCtx<D> {
    fn clone(&self) -> Self { Self { app: self.app.clone(), definition_id: self.definition_id.clone() } }
}
impl<D> RenderCtx<D> {
    pub(crate) fn new(app: App<D>, definition_id: Arc<str>) -> Self { Self { app, definition_id } }
}

/// Interaction-specific view of the App with correctly scoped Discord response operations.
pub struct Ctx<D = Arc<()>> {
    pub app: App<D>,
    pub interaction: Arc<Interaction>,
    pub http: Arc<Http>,
    pub selection: Option<Selection>,
    acknowledged: Arc<AtomicBool>,
}
impl<D: Clone + Send + Sync + 'static> Ctx<D> {
    pub(crate) fn new(
        app: App<D>,
        interaction: Arc<Interaction>,
        selection: Option<Selection>,
    ) -> Self {
        let http = app.http.clone();
        Self { app, interaction, http, selection, acknowledged: Arc::new(AtomicBool::new(false)) }
    }

    fn credentials(&self) -> anyhow::Result<(serenity::all::InteractionId, String)> {
        match self.interaction.as_ref() {
            Interaction::Command(value) => Ok((value.id, value.token.to_string())),
            Interaction::Autocomplete(value) => Ok((value.id, value.token.to_string())),
            Interaction::Component(value) => Ok((value.id, value.token.to_string())),
            Interaction::Modal(value) => Ok((value.id, value.token.to_string())),
            _ => anyhow::bail!("This Discord interaction cannot receive an application response"),
        }
    }

    fn acknowledge_once(&self) -> anyhow::Result<()> {
        self.acknowledged.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| anyhow::anyhow!("This interaction has already received an initial response"))
    }

    fn outgoing_attachments(message: Vec<crate::page::OutgoingAttachment>) -> Vec<CreateAttachment<'static>> {
        message.into_iter().map(|attachment| {
            CreateAttachment::bytes((*attachment.bytes).clone(), attachment.filename)
        }).collect()
    }

    /// Returns the interacting user when the interaction type has one.
    pub fn user_id(&self) -> Option<serenity::all::UserId> {
        match self.interaction.as_ref() {
            Interaction::Command(value) | Interaction::Autocomplete(value) => Some(value.user.id),
            Interaction::Component(value) => Some(value.user.id),
            Interaction::Modal(value) => Some(value.user.id),
            _ => None,
        }
    }

    /// Returns the guild containing the interaction, or no value in a DM.
    pub fn guild_id(&self) -> Option<serenity::all::GuildId> { self.interaction.guild_id() }

    /// Returns the channel containing the interaction when available.
    pub fn channel_id(&self) -> Option<serenity::all::ChannelId> {
        use serenity::all::ChannelId;
        match self.interaction.as_ref() {
            Interaction::Command(value) | Interaction::Autocomplete(value) =>
                Some(ChannelId::new(value.channel_id.get())),
            Interaction::Component(value) => Some(ChannelId::new(value.channel_id.get())),
            Interaction::Modal(value) => Some(ChannelId::new(value.channel_id.get())),
            _ => None,
        }
    }

    /// Sends the initial Channel Message With Source interaction response.
    pub fn reply(&self, content: impl Into<String>) -> BoxFuture<'_, anyhow::Result<()>> {
        let content = content.into();
        Box::pin(async move {
            self.acknowledge_once()?;
            let (id, token) = self.credentials()?;
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content(content)
            );
            response.execute(&self.http, id, &token).await?;
            Ok(())
        })
    }

    /// Defers the initial response with Discord's Deferred Channel Message response.
    pub fn defer(&self) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move {
            self.acknowledge_once()?;
            let (id, token) = self.credentials()?;
            CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new())
                .execute(&self.http, id, &token).await?;
            Ok(())
        })
    }

    /// Updates the message containing the component that triggered this interaction.
    pub fn edit_message(&self, message: RenderedMessage) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move {
            self.acknowledge_once()?;
            let (id, token) = self.credentials()?;
            anyhow::ensure!(matches!(self.interaction.as_ref(), Interaction::Component(_)),
                "edit_message requires a Component interaction");
            let (components, attachments, _) = message.into_parts();
            let mut payload = CreateInteractionResponseMessage::new()
                .components(components)
                .flags(serenity::all::MessageFlags::IS_COMPONENTS_V2);
            payload = payload.add_files(Self::outgoing_attachments(attachments));
            CreateInteractionResponse::UpdateMessage(payload).execute(&self.http, id, &token).await?;
            Ok(())
        })
    }

    /// Edits the original response associated with this interaction.
    pub fn edit_original_response(&self, message: RenderedMessage) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move {
            let (_, token) = self.credentials()?;
            let (components, attachments, _) = message.into_parts();
            let edit_attachments = Self::outgoing_attachments(attachments)
                .into_iter().fold(EditAttachments::new(), |acc, file| acc.add(file));
            let payload = EditInteractionResponse::new()
                .components(components)
                .flags(serenity::all::MessageFlags::IS_COMPONENTS_V2)
                .attachments(edit_attachments);
            payload.execute(&self.http, &token).await?;
            Ok(())
        })
    }

    /// Creates a separate webhook follow-up message.
    pub fn followup(&self, message: RenderedMessage) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move {
            let (_, token) = self.credentials()?;
            let (components, attachments, _) = message.into_parts();
            let mut payload = CreateInteractionResponseFollowup::new()
                .components(components)
                .flags(serenity::all::MessageFlags::IS_COMPONENTS_V2);
            payload = payload.add_files(Self::outgoing_attachments(attachments));
            payload.execute(&self.http, None, &token).await?;
            Ok(())
        })
    }

    /// Deletes the original interaction webhook response.
    pub fn delete_response(&self) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move {
            let (_, token) = self.credentials()?;
            self.http.delete_original_interaction_response(&token).await?;
            Ok(())
        })
    }

    /// Opens a Modal as the initial Modal interaction response.
    pub fn open_modal(&self, modal: ModalRender) -> BoxFuture<'_, anyhow::Result<()>> {
        Box::pin(async move {
            self.acknowledge_once()?;
            let (id, token) = self.credentials()?;
            let payload = CreateInteractionResponse::Modal(
                CreateModal::new(modal.custom_id, modal.title).components(modal.components)
            );
            payload.execute(&self.http, id, &token).await?;
            Ok(())
        })
    }

    /// Borrows the typed selection payload, if this interaction carried one.
    pub fn selected(&self) -> Option<&Selection> { self.selection.as_ref() }
}
