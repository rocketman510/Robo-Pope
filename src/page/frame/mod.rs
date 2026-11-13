//! Dynamic, heterogeneous region with optional automatic pagination.
pub mod element;
pub mod pagination;

pub use element::{Element,FrameIdentity};

use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture,RenderCtx};

pub struct Frame<T> {
    pub id:String,
    pub(crate) resolver:Arc<dyn Fn(RenderCtx,Arc<T>)
        ->BoxFuture<'static,anyhow::Result<Vec<Element<T>>>>+Send+Sync>,
    pub paginate:bool,
}
impl<T:Send+Sync+'static,> Frame<T> {
    pub fn new<F,Fut>(id:impl Into<String>,resolver:F)->Self
    where
        F:Fn(RenderCtx,Arc<T>)->Fut+Send+Sync+'static,
        Fut:Future<Output=anyhow::Result<Vec<Element<T>>>>+Send+'static,
    {
        todo!("store resolver and stable developer-defined Frame ID")
    }
    pub fn paginate(mut self)->Self { self.paginate=true; self }
}
