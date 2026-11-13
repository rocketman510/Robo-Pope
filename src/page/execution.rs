//! Explicit callbacks for interactive components. Dynamic values are not executions.
use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture, Ctx};

/// Explicit interaction callback associated with a stable developer-defined ID.
///
/// `T` is the containing Page or Modal data. `I` is the interaction payload:
/// `()` for a Button and a typed selection or Modal submission for inputs.
pub struct Execution<T, I = ()> {
    /// Stable ID within the containing Page or Modal definition.
    /// Stable developer-defined ID within the containing definition.
    pub id: String,
    pub(crate) callback: Arc<dyn Fn(Ctx, T, I)
        -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,
}

impl<T> Execution<T> {
    /// Creates an execution with no additional interaction payload.
    /// The callback receives the interaction context and reconstructed Page data.
    pub fn new<F, Fut>(id: impl Into<String>, callback: F) -> Self
    where
        F: Fn(Ctx, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        T: Send + 'static,
        {
        Self {
            id: id.into(),
            callback: Arc::new(move |ctx, data| Box::pin(callback(ctx, data))),
        }
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
        Self {
            id: id.into(),
            callback: Arc::new(move |ctx, data, input| Box::pin(callback(ctx, data, input))),
        }
    }
}

#[derive(Debug, Clone)]
/// Typed selection payload produced by supported Discord select menus.
pub enum Selection {
    Strings(Vec<String>),
    Users(Vec<SelectedUser>),
    Roles(Vec<SelectedRole>),
    Mentionables(Vec<SelectedMentionable>),
    Channels(Vec<SelectedChannel>),
}
#[derive(Debug, Clone)]
/// Selected Discord User ID and optional resolved User object.
pub struct SelectedUser {
    /// Discord ID of the selected user.
    pub id: serenity::all::UserId,
    pub resolved: Option<Arc<serenity::all::User>>,
}
#[derive(Debug, Clone)]
/// Selected Discord Role ID and optional resolved Role object.
pub struct SelectedRole {
    pub id: serenity::all::RoleId,
    pub resolved: Option<Arc<serenity::all::Role>>,
}
#[derive(Debug, Clone)]
/// A selected User or Role from a Mentionable Select.
pub enum SelectedMentionable {
    User(SelectedUser),
    Role(SelectedRole),
}
#[derive(Debug, Clone)]
/// Selected Discord Channel ID and optional resolved guild channel.
pub struct SelectedChannel {
    pub id: serenity::all::ChannelId,
    pub resolved: Option<Arc<serenity::all::GuildChannel>>,
}
