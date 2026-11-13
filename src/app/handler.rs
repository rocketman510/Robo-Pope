//! Gateway adapter: App owns framework state; AppHandler receives gateway events.
use serenity::{all::{Context,EventHandler,FullEvent},async_trait};
use crate::app::App;

pub struct AppHandler<E> { pub app:App<E> }
impl<E> AppHandler<E> {
    pub fn new(app:App<E>)->Self { Self { app } }
}
#[async_trait]
impl<E:Send+Sync+'static> EventHandler for AppHandler<E> {
    async fn dispatch(&self,_ctx:&Context,event:&FullEvent) {
        if let FullEvent::InteractionCreate { interaction,.. } = event {
            todo!("dispatch InteractionCreate through App and log contextual anyhow errors")
        }
    }
}
