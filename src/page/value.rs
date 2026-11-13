//! Static and asynchronous dynamic values share one property API.
use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture,RenderCtx};

/// Explicit adapter for an asynchronous property callback. It avoids
/// overlapping blanket trait implementations for arbitrary closures.
pub struct Dynamic<F>(pub F);
pub fn dynamic<F>(callback:F)->Dynamic<F> { Dynamic(callback) }

pub enum Value<V,T,E> {
    Static(V),
    Dynamic(Arc<dyn Fn(RenderCtx<E>,Arc<T>)
        ->BoxFuture<'static,anyhow::Result<V>>+Send+Sync>),
}
pub trait IntoValue<V,T,E> { fn into_value(self)->Value<V,T,E>; }

impl<T,E> IntoValue<String,T,E> for String {
    fn into_value(self)->Value<String,T,E> { Value::Static(self) }
}
impl<T,E> IntoValue<String,T,E> for &'static str {
    fn into_value(self)->Value<String,T,E> { Value::Static(self.to_owned()) }
}
impl<T,E> IntoValue<bool,T,E> for bool {
    fn into_value(self)->Value<bool,T,E> { Value::Static(self) }
}
impl<T,E> IntoValue<u32,T,E> for u32 {
    fn into_value(self)->Value<u32,T,E> { Value::Static(self) }
}
impl<T,E> IntoValue<super::component::ButtonStyle,T,E> for super::component::ButtonStyle {
    fn into_value(self)->Value<super::component::ButtonStyle,T,E> { Value::Static(self) }
}
impl<T,E> IntoValue<super::component::SeparatorSpacing,T,E> for super::component::SeparatorSpacing {
    fn into_value(self)->Value<super::component::SeparatorSpacing,T,E> { Value::Static(self) }
}
impl<V,T,E,F,Fut> IntoValue<V,T,E> for Dynamic<F>
where
    F:Fn(RenderCtx<E>,Arc<T>)->Fut+Send+Sync+'static,
    Fut:Future<Output=anyhow::Result<V>>+Send+'static,
    V:Send+Sync+'static,T:Send+Sync+'static,E:Send+Sync+'static,
{
    fn into_value(self)->Value<V,T,E> {
        Value::Dynamic(Arc::new(move |ctx,data|Box::pin((self.0)(ctx,data))))
    }
}

pub enum Slot<V,T,E> { Empty, Value(Value<V,T,E>) }
pub trait IntoSlot<V,T,E> { fn into_slot(self)->Slot<V,T,E>; }

impl<V,T,E> IntoSlot<V,T,E> for Option<Value<V,T,E>> {
    fn into_slot(self)->Slot<V,T,E> {
        match self { Some(value)=>Slot::Value(value),None=>Slot::Empty }
    }
}
impl<T,E> IntoSlot<String,T,E> for String {
    fn into_slot(self)->Slot<String,T,E> { Slot::Value(Value::Static(self)) }
}
impl<T,E> IntoSlot<String,T,E> for &'static str {
    fn into_slot(self)->Slot<String,T,E> { Slot::Value(Value::Static(self.to_owned())) }
}
impl<V,T,E,F> IntoSlot<V,T,E> for Dynamic<F>
where Dynamic<F>:IntoValue<V,T,E>
{
    fn into_slot(self)->Slot<V,T,E> { Slot::Value(self.into_value()) }
}

pub async fn resolve<V,T,E>(
    value:&Value<V,T,E>,ctx:RenderCtx<E>,data:Arc<T>
)->anyhow::Result<V>
where V:Clone
{
    match value {
        Value::Static(value)=>Ok(value.clone()),
        Value::Dynamic(callback)=>callback(ctx,data).await,
    }
}
