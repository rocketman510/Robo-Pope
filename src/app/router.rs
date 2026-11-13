//! Route interactions by stored identity, never by visual component position.
use crate::app::App;
use serenity::all::Interaction;
use std::marker::PhantomData;

pub struct InteractionRouter<E> { marker:PhantomData<fn()->E> }
impl<E> Clone for InteractionRouter<E> {
    fn clone(&self)->Self { Self { marker:PhantomData } }
}
impl<E> Default for InteractionRouter<E> {
    fn default()->Self { Self { marker:PhantomData } }
}
impl<E:Send+Sync+'static> InteractionRouter<E> {
    pub async fn dispatch(&self,app:&App<E>,interaction:Interaction)->anyhow::Result<()> {
        todo!("identify interaction kind and delegate to its route")
    }
    pub async fn dispatch_component(&self,app:&App<E>,interaction:Interaction)->anyhow::Result<()> {
        todo!("resolve Store record, definition, execution, Page data, and selection payload")
    }
    pub async fn dispatch_modal_submit(&self,app:&App<E>,interaction:Interaction)->anyhow::Result<()> {
        todo!("resolve Modal data and pass ModalSubmission separately")
    }
}
