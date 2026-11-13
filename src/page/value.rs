//! Static and asynchronous dynamic values share one property API.
use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture,RenderCtx};

/// Explicit adapter for an asynchronous property callback. It avoids
/// overlapping blanket trait implementations for arbitrary closures.
pub struct Dynamic<F>(pub F);
pub fn dynamic<F>(callback:F)->Dynamic<F> { Dynamic(callback) }

pub enum Value<V,T> {
    Static(V),
    Dynamic(Arc<dyn Fn(RenderCtx,Arc<T>)
        ->BoxFuture<'static,anyhow::Result<V>>+Send+Sync>),
}
pub trait IntoValue<V,T> { fn into_value(self)->Value<V,T>; }

impl<T> IntoValue<String,T> for String {
    fn into_value(self)->Value<String,T> { Value::Static(self) }
}
impl<T> IntoValue<String,T> for &'static str {
    fn into_value(self)->Value<String,T> { Value::Static(self.to_owned()) }
}
impl<T> IntoValue<bool,T> for bool {
    fn into_value(self)->Value<bool,T> { Value::Static(self) }
}
impl<T> IntoValue<u32,T> for u32 {
    fn into_value(self)->Value<u32,T> { Value::Static(self) }
}
impl<T> IntoValue<super::component::ButtonStyle,T> for super::component::ButtonStyle {
    fn into_value(self)->Value<super::component::ButtonStyle,T> { Value::Static(self) }
}
impl<T> IntoValue<super::component::SeparatorSpacing,T> for super::component::SeparatorSpacing {
    fn into_value(self)->Value<super::component::SeparatorSpacing,T> { Value::Static(self) }
}
impl<V,T,F,Fut> IntoValue<V,T> for Dynamic<F>
where
    F:Fn(RenderCtx,Arc<T>)->Fut+Send+Sync+'static,
    Fut:Future<Output=anyhow::Result<V>>+Send+'static,
    V:Send+Sync+'static,T:Send+Sync+'static,{
    fn into_value(self)->Value<V,T> {
        Value::Dynamic(Arc::new(move |ctx,data|Box::pin((self.0)(ctx,data))))
    }
}

pub enum Slot<V,T> { Empty, Value(Value<V,T>) }
pub trait IntoSlot<V,T> { fn into_slot(self)->Slot<V,T>; }

impl<V,T> IntoSlot<V,T> for Option<Value<V,T>> {
    fn into_slot(self)->Slot<V,T> {
        match self { Some(value)=>Slot::Value(value),None=>Slot::Empty }
    }
}
impl<T> IntoSlot<String,T> for String {
    fn into_slot(self)->Slot<String,T> { Slot::Value(Value::Static(self)) }
}
impl<T> IntoSlot<String,T> for &'static str {
    fn into_slot(self)->Slot<String,T> { Slot::Value(Value::Static(self.to_owned())) }
}
impl<V,T,F> IntoSlot<V,T> for Dynamic<F>
where Dynamic<F>:IntoValue<V,T>
{
    fn into_slot(self)->Slot<V,T> { Slot::Value(self.into_value()) }
}

pub async fn resolve<V,T>(
    value:&Value<V,T>,ctx:RenderCtx,data:Arc<T>
)->anyhow::Result<V>
where V:Clone
{
    match value {
        Value::Static(value)=>Ok(value.clone()),
        Value::Dynamic(callback)=>callback(ctx,data).await,
    }
}
