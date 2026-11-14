//! Static and asynchronous render-time values shared by Page properties.
use std::future::Future;
use std::sync::Arc;

use crate::app::{BoxFuture, RenderCtx};

pub type Dynamic<V, T, D = Arc<()>> = Arc<
    dyn Fn(RenderCtx<D>, Arc<T>) -> BoxFuture<'static, anyhow::Result<V>> + Send + Sync + 'static,
>;

pub enum Value<V, T, D = Arc<()>> {
    Static(V),
    Dynamic(Dynamic<V, T, D>),
}

impl<V: Clone, T, D> Clone for Value<V, T, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(value) => Self::Static(value.clone()),
            Self::Dynamic(value) => Self::Dynamic(value.clone()),
        }
    }
}

/// Explicit adapter for the default App database type.
pub fn dynamic<V, T, F, Fut>(callback: F) -> Value<V, T>
where
    F: Fn(RenderCtx<Arc<()>>, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
{
    Value::Dynamic(Arc::new(move |ctx, data| Box::pin(callback(ctx, data))))
}

/// Explicit adapter for applications with a concrete database type.
pub fn dynamic_for<D, V, T, F, Fut>(callback: F) -> Value<V, T, D>
where
    F: Fn(RenderCtx<D>, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    Value::Dynamic(Arc::new(move |ctx, data| Box::pin(callback(ctx, data))))
}

pub trait IntoValue<V, T, D = Arc<()>> {
    fn into_value(self) -> Value<V, T, D>;
}

impl<V, T, D> IntoValue<V, T, D> for Value<V, T, D> {
    fn into_value(self) -> Value<V, T, D> { self }
}

impl<V, T, D> IntoValue<V, T, D> for Dynamic<V, T, D> {
    fn into_value(self) -> Value<V, T, D> { Value::Dynamic(self) }
}

impl<V, T, D, F, Fut> IntoValue<V, T, D> for F
where
    F: Fn(RenderCtx<D>, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    fn into_value(self) -> Value<V, T, D> {
        Value::Dynamic(Arc::new(move |ctx, data| Box::pin(self(ctx, data))))
    }
}

impl<T, D> IntoValue<String, T, D> for String {
    fn into_value(self) -> Value<String, T, D> { Value::Static(self) }
}

impl<'a, T, D> IntoValue<String, T, D> for &'a str {
    fn into_value(self) -> Value<String, T, D> { Value::Static(self.to_owned()) }
}

impl<T, D> IntoValue<Option<String>, T, D> for Option<String> {
    fn into_value(self) -> Value<Option<String>, T, D> { Value::Static(self) }
}

macro_rules! impl_static_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl<T, D> IntoValue<$ty, T, D> for $ty {
                fn into_value(self) -> Value<$ty, T, D> { Value::Static(self) }
            }
        )+
    };
}
impl_static_value!(bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, f32, f64);

pub enum Slot<V, T, D = Arc<()>> {
    Empty,
    Value(Value<V, T, D>),
}

impl<V: Clone, T, D> Clone for Slot<V, T, D> {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Value(value) => Self::Value(value.clone()),
        }
    }
}

pub trait IntoSlot<V, T, D = Arc<()>> {
    fn into_slot(self) -> Slot<V, T, D>;
}

impl<V, T, D> IntoSlot<V, T, D> for Value<V, T, D> {
    fn into_slot(self) -> Slot<V, T, D> { Slot::Value(self) }
}

impl<V, T, D> IntoSlot<V, T, D> for Dynamic<V, T, D> {
    fn into_slot(self) -> Slot<V, T, D> { Slot::Value(Value::Dynamic(self)) }
}

impl<V, T, D, F, Fut> IntoSlot<V, T, D> for F
where
    F: Fn(RenderCtx<D>, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
    D: Send + Sync + 'static,
{
    fn into_slot(self) -> Slot<V, T, D> { Slot::Value(self.into_value()) }
}

impl<T, D> IntoSlot<String, T, D> for String {
    fn into_slot(self) -> Slot<String, T, D> { Slot::Value(Value::Static(self)) }
}

impl<'a, T, D> IntoSlot<String, T, D> for &'a str {
    fn into_slot(self) -> Slot<String, T, D> { Slot::Value(Value::Static(self.to_owned())) }
}

impl<T, D> IntoSlot<String, T, D> for Option<String> {
    fn into_slot(self) -> Slot<String, T, D> {
        match self {
            Some(value) => Slot::Value(Value::Static(value)),
            None => Slot::Empty,
        }
    }
}

pub async fn resolve<V, T, D>(
    value: &Value<V, T, D>,
    ctx: RenderCtx<D>,
    data: Arc<T>,
) -> anyhow::Result<V>
where
    V: Clone + Send + 'static,
    T: Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    match value {
        Value::Static(value) => Ok(value.clone()),
        Value::Dynamic(callback) => callback(ctx, data).await,
    }
}
