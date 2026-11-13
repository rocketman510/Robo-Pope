//! Explicit callbacks for interactive components. Dynamic values are not executions.
use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture, Ctx};

pub struct Execution<T, I = ()> {
    pub id: String,
    pub(crate) callback: Arc<dyn Fn(Ctx, T, I)
        -> BoxFuture<'static, anyhow::Result<()>> + Send + Sync>,
}

impl<T> Execution<T> {
    pub fn new<F, Fut>(id: impl Into<String>, callback: F) -> Self
    where
        F: Fn(Ctx, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        T: Send + 'static,
        {
        todo!("store two-argument execution callback behind boxed future")
    }
}
impl<T, I> Execution<T, I> {
    pub fn with_input<F, Fut>(id: impl Into<String>, callback: F) -> Self
    where
        F: Fn(Ctx, T, I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
        T: Send + 'static,
        I: Send + 'static,
    {
        todo!("store callback with its distinct interaction payload")
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
pub enum SelectedMentionable {
    User(SelectedUser),
    Role(SelectedRole),
}
#[derive(Debug, Clone)]
pub struct SelectedChannel {
    pub id: serenity::all::ChannelId,
    pub resolved: Option<Arc<serenity::all::GuildChannel>>,
}
