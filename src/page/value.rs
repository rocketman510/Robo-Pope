//! Static and asynchronous render-time values shared by Page properties.
use std::future::Future;
use std::sync::Arc;
use crate::app::{BoxFuture, RenderCtx};

pub type Dynamic<V, T> = Arc<
    dyn Fn(RenderCtx, Arc<T>) -> BoxFuture<'static, anyhow::Result<V>> + Send + Sync + 'static,
>;

pub enum Value<V, T> { Static(V), Dynamic(Dynamic<V, T>) }
impl<V: Clone, T> Clone for Value<V, T> {
    fn clone(&self) -> Self {
        match self { Self::Static(value) => Self::Static(value.clone()), Self::Dynamic(value) => Self::Dynamic(value.clone()) }
    }
}

/// Explicit adapter for callbacks. Direct async closures are accepted by the same API.
pub fn dynamic<V, T, F, Fut>(callback: F) -> Dynamic<V, T>
where
    F: Fn(RenderCtx, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
{
    Arc::new(move |ctx, data| Box::pin(callback(ctx, data)))
}

pub trait IntoValue<V, T> { fn into_value(self) -> Value<V, T>; }
impl<V, T, F, Fut> IntoValue<V, T> for F
where
    F: Fn(RenderCtx, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
{
    fn into_value(self) -> Value<V, T> {
        Value::Dynamic(Arc::new(move |ctx, data| Box::pin(self(ctx, data))))
    }
}
impl<T> IntoValue<String, T> for String {
    fn into_value(self) -> Value<String, T> { Value::Static(self) }
}
impl<'a, T> IntoValue<String, T> for &'a str {
    fn into_value(self) -> Value<String, T> { Value::Static(self.to_owned()) }
}
macro_rules! impl_static_value {
    ($($ty:ty),+ $(,)?) => {
        $(impl<T> IntoValue<$ty, T> for $ty {
            fn into_value(self) -> Value<$ty, T> { Value::Static(self) }
        })+
    };
}
impl_static_value!(bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, f32, f64);

pub enum Slot<V, T> { Empty, Value(Value<V, T>) }
impl<V: Clone, T> Clone for Slot<V, T> {
    fn clone(&self) -> Self {
        match self { Self::Empty => Self::Empty, Self::Value(value) => Self::Value(value.clone()) }
    }
}
pub trait IntoSlot<V, T> { fn into_slot(self) -> Slot<V, T>; }
impl<V, T, F, Fut> IntoSlot<V, T> for F
where
    F: Fn(RenderCtx, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
{
    fn into_slot(self) -> Slot<V, T> { Slot::Value(self.into_value()) }
}
impl<T> IntoSlot<String, T> for String {
    fn into_slot(self) -> Slot<String, T> { Slot::Value(Value::Static(self)) }
}
impl<'a, T> IntoSlot<String, T> for &'a str {
    fn into_slot(self) -> Slot<String, T> { Slot::Value(Value::Static(self.to_owned())) }
}
impl<T> IntoSlot<String, T> for Option<String> {
    fn into_slot(self) -> Slot<String, T> {
        match self { Some(value) => Slot::Value(Value::Static(value)), None => Slot::Empty }
    }
}

pub async fn resolve<V, T>(value: &Value<V, T>, ctx: RenderCtx, data: Arc<T>) -> anyhow::Result<V>
where V: Clone + Send + 'static, T: Send + Sync + 'static {
    match value { Value::Static(value) => Ok(value.clone()), Value::Dynamic(callback) => callback(ctx, data).await }
}
