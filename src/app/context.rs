//! Temporary contexts supplied to dynamic values and interactive executions.
use std::sync::Arc;
use serenity::all::{Http, Interaction};
use crate::app::{App, BoxFuture};
use crate::page::{ModalRender, RenderedMessage, Selection};

/// Render-time context for resolving dynamic Page properties and Frame contents.
///
/// It owns a cheap shared handle to the App and identifies the definition being
/// rendered. It is temporary and is never persisted.
pub struct RenderCtx {
    /// Shared access to all application-owned state.
    pub app: App,
    /// ID of the Page or Modal definition currently being processed.
    pub definition_id: Arc<str>,
}
impl Clone for RenderCtx {
    fn clone(&self) -> Self {
        Self { app: self.app.clone(), definition_id: self.definition_id.clone() }
    }
}
impl RenderCtx {
    pub(crate) fn new(app: App, definition_id: Arc<str>) -> Self {
        Self { app, definition_id }
    }
}

/// Interaction-specific view of the monolithic App.
///
/// Every interactive callback receives this context. It links interaction
/// details and response operations to the same App that owns application state.
pub struct Ctx {
    /// Shared access to application state and registered definitions.
    pub app: App,
    /// The Discord interaction that caused this execution.
    pub interaction: Arc<Interaction>,
    /// Serenity HTTP handle used to respond to the interaction.
    pub http: Arc<Http>,
    /// Typed values selected by a selection-menu interaction, when present.
    pub selection: Option<Selection>,
}
impl Ctx {
    /// Builds a temporary context from the current interaction and resolved selection.
    pub(crate) fn new(app: App, interaction: Arc<Interaction>, selection: Option<Selection>) -> Self {
        let http = app.http.clone();
        Self { app, interaction, http, selection }
    }
    /// Returns the interacting user when the interaction type has one.
    pub fn user_id(&self) -> Option<serenity::all::UserId> {
        use serenity::all::Interaction;
        match self.interaction.as_ref() {
            Interaction::Command(value) | Interaction::Autocomplete(value) => Some(value.user.id),
            Interaction::Component(value) => Some(value.user.id),
            Interaction::Modal(value) => Some(value.user.id),
            Interaction::Ping(_) => None,
            _ => None,
        }
    }
    /// Returns the guild containing the interaction, or no value in a DM.
    pub fn guild_id(&self) -> Option<serenity::all::GuildId> {
        self.interaction.guild_id()
    }
    /// Returns the channel containing the interaction when available.
    pub fn channel_id(&self) -> Option<serenity::all::ChannelId> {
        use serenity::all::{ChannelId, Interaction};
        match self.interaction.as_ref() {
            Interaction::Command(value) | Interaction::Autocomplete(value) => {
                Some(ChannelId::new(value.channel_id.get()))
            }
            Interaction::Component(value) => Some(ChannelId::new(value.channel_id.get())),
            Interaction::Modal(value) => Some(ChannelId::new(value.channel_id.get())),
            Interaction::Ping(_) => None,
            _ => None,
        }
    }
    /// Sends the initial response to this interaction, not a follow-up message.
    pub fn reply(&self, content: impl Into<String>) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("send initial interaction response")
    }
    /// Defers the initial response so the callback can finish processing first.
    pub fn defer(&self) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("defer initial response")
    }
    /// Edits the message that contains the interactive component.
    pub fn edit_message(&self, message: RenderedMessage) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("edit component message")
    }
    /// Edits the original response associated with this interaction.
    pub fn edit_original_response(&self, message: RenderedMessage) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("edit original response")
    }
    /// Sends a follow-up response after the initial response has been handled.
    pub fn followup(&self, message: RenderedMessage) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("send follow-up response")
    }
    /// Deletes the response associated with this interaction.
    pub fn delete_response(&self) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("delete interaction response")
    }
    /// Opens a Modal as the initial response to this interaction.
    pub fn open_modal(&self, modal: ModalRender) -> BoxFuture<'_, anyhow::Result<()>> {
        todo!("open Modal as initial interaction response")
    }
    /// Borrows the typed selection payload, if this interaction carried one.
    pub fn selected(&self) -> Option<&Selection> { self.selection.as_ref() }
}
