//! Explicit callbacks for interactive components. Dynamic values are not executions.
use std::future::Future;
use std::sync::Arc;
use serde::de::DeserializeOwned;

use crate::app::{App, BoxFuture, Ctx};
use crate::app::executions::{ErasedCallback, ExecutionKey, InteractionPayload};
use crate::page::ModalSubmission;

pub struct Execution<T, I = ()> {
    /// Stable ID within the containing Page or Modal definition.
    pub id: String,
    pub(crate) callback: Arc<dyn Fn(Ctx, T, I)
        -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,
}

/// Converts the erased router payload back into the callback's declared input.
pub(crate) trait FromInteractionPayload: Sized + Send + 'static {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self>;
}
impl FromInteractionPayload for () {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::None => Ok(()), _ => anyhow::bail!("Unexpected selection payload for Button execution") }
    }
}
impl FromInteractionPayload for Vec<String> {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::Selection(Selection::Strings(v)) => Ok(v), _ => anyhow::bail!("Expected String Select payload") }
    }
}
impl FromInteractionPayload for Vec<SelectedUser> {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::Selection(Selection::Users(v)) => Ok(v), _ => anyhow::bail!("Expected User Select payload") }
    }
}
impl FromInteractionPayload for Vec<SelectedRole> {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::Selection(Selection::Roles(v)) => Ok(v), _ => anyhow::bail!("Expected Role Select payload") }
    }
}
impl FromInteractionPayload for Vec<SelectedMentionable> {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::Selection(Selection::Mentionables(v)) => Ok(v), _ => anyhow::bail!("Expected Mentionable Select payload") }
    }
}
impl FromInteractionPayload for Vec<SelectedChannel> {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::Selection(Selection::Channels(v)) => Ok(v), _ => anyhow::bail!("Expected Channel Select payload") }
    }
}
impl FromInteractionPayload for ModalSubmission {
    fn from_payload(payload: InteractionPayload) -> anyhow::Result<Self> {
        match payload { InteractionPayload::Modal(v) => Ok(v), _ => anyhow::bail!("Expected Modal submission payload") }
    }
}

impl<T> Execution<T> {
    /// Creates an execution with no additional interaction payload.
    pub fn new<F, Fut>(id: impl Into<String>, callback: F) -> Self
    where
        F: Fn(Ctx, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        T: Send + 'static,
    {
        Self { id: id.into(), callback: Arc::new(move |ctx, data, _input| Box::pin(callback(ctx, data))) }
    }
}
impl<T, I> Execution<T, I> {
    /// Creates an execution whose callback also receives a typed interaction payload.
    pub fn with_input<F, Fut>(id: impl Into<String>, callback: F) -> Self
    where
        F: Fn(Ctx, T, I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        T: Send + 'static,
        I: Send + 'static,
    {
        Self { id: id.into(), callback: Arc::new(move |ctx, data, input| Box::pin(callback(ctx, data, input))) }
    }

    /// Registers a type-erased adapter for this callback in the App.
    /// Frame identity is part of the process-local lookup key.
    pub(crate) fn register(
        &self,
        app: &App,
        definition_id: &str,
        frame_identity: Option<&str>,
    ) -> anyhow::Result<()>
    where
        T: DeserializeOwned + Send + 'static,
        I: FromInteractionPayload,
    {
        let execution_id = match frame_identity {
            Some(identity) => format!("{identity}::{}", self.id),
            None => self.id.clone(),
        };
        let key = ExecutionKey { definition_id: definition_id.to_owned(), execution_id };
        let callback = self.callback.clone();
        let erased: ErasedCallback = Arc::new(move |ctx, json, payload| {
            let callback = callback.clone();
            Box::pin(async move {
                let data: T = serde_json::from_value(json)
                    .map_err(|error| anyhow::anyhow!("Deserializing execution data: {error}"))?;
                let input = I::from_payload(payload)?;
                callback(ctx, data, input).await
            })
        });
        app.executions.insert_or_replace(key, erased)
    }
}

#[derive(Debug, Clone)]
pub enum Selection {
    Strings(Vec<String>),
    Users(Vec<SelectedUser>),
    Roles(Vec<SelectedRole>),
    Mentionables(Vec<SelectedMentionable>),
    Channels(Vec<SelectedChannel>),
}
#[derive(Debug, Clone)]
pub struct SelectedUser {
    pub id: serenity::all::UserId,
    pub resolved: Option<Arc<serenity::all::User>>,
}
#[derive(Debug, Clone)]
pub struct SelectedRole {
    pub id: serenity::all::RoleId,
    pub resolved: Option<Arc<serenity::all::Role>>,
}
#[derive(Debug, Clone)]
pub enum SelectedMentionable { User(SelectedUser), Role(SelectedRole) }
#[derive(Debug, Clone)]
pub struct SelectedChannel {
    pub id: serenity::all::ChannelId,
    pub resolved: Option<Arc<serenity::all::GuildChannel>>,
}
