//! Temporary contexts supplied to dynamic values and interactive executions.
use std::sync::Arc;
use serenity::all::{Http,Interaction};
use crate::app::{App,BoxFuture};
use crate::page::{ModalRender,RenderedMessage,Selection};

pub struct RenderCtx<E=()> {
    pub app:App<E>,
    pub definition_id:Arc<str>,
}
impl<E> Clone for RenderCtx<E> {
    fn clone(&self)->Self {
        Self { app:self.app.clone(),definition_id:self.definition_id.clone() }
    }
}
impl<E> RenderCtx<E> {
    pub(crate) fn new(app:App<E>,definition_id:Arc<str>)->Self {
        Self { app,definition_id }
    }
}

pub struct Ctx<E=()> {
    pub app:App<E>,
    pub interaction:Arc<Interaction>,
    pub http:Arc<Http>,
    pub selection:Option<Selection>,
}
impl<E:Send+Sync+'static> Ctx<E> {
    pub(crate) fn new(app:App<E>,interaction:Arc<Interaction>,selection:Option<Selection>)->Self {
        todo!("construct interaction context from the actual Serenity payload")
    }
    pub fn user_id(&self)->Option<serenity::all::UserId> {
        todo!("return interacting user when present")
    }
    pub fn guild_id(&self)->Option<serenity::all::GuildId> {
        todo!("return guild ID when available")
    }
    pub fn channel_id(&self)->Option<serenity::all::ChannelId> {
        todo!("return channel ID when available")
    }
    pub fn reply(&self,content:impl Into<String>)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("send initial interaction response")
    }
    pub fn defer(&self)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("defer initial response")
    }
    pub fn edit_message(&self,message:RenderedMessage)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("edit component message")
    }
    pub fn edit_original_response(&self,message:RenderedMessage)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("edit original response")
    }
    pub fn followup(&self,message:RenderedMessage)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("send follow-up response")
    }
    pub fn delete_response(&self)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("delete interaction response")
    }
    pub fn open_modal(&self,modal:ModalRender)->BoxFuture<'_,anyhow::Result<()>> {
        todo!("open Modal as initial interaction response")
    }
    pub fn selected(&self)->Option<&Selection> { self.selection.as_ref() }
}
