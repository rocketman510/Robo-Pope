//! Static and asynchronous render-time values shared by Page properties.
use std::future::Future;
use std::sync::Arc;

use crate::app::{BoxFuture, RenderCtx};

/// A property that is either a concrete value or a callback evaluated while rendering.
pub enum Value<V, T> {
    /// A value known when the Page definition is built.
    Static(V),
    /// A callback evaluated with the current App-linked render context and Page data.
    Dynamic(Dynamic<V, T>),
}

impl<V: Clone, T> Clone for Value<V, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Static(value) => Self::Static(value.clone()),
            Self::Dynamic(value) => Self::Dynamic(value.clone()),
        }
    }
}

/// An asynchronous property resolver.
///
/// The callback receives a temporary RenderCtx and shared access to the Page's
/// data. It returns the property value or an anyhow error. The callback itself
/// is retained by the Page definition and is never serialized.
pub struct Dynamic<V, T> {
    resolver: Arc<
        dyn Fn(RenderCtx, Arc<T>) -> BoxFuture<'static, anyhow::Result<V>>
            + Send
            + Sync
            + 'static,
    >,
}

impl<V, T> Clone for Dynamic<V, T> {
    fn clone(&self) -> Self {
        Self { resolver: self.resolver.clone() }
    }
}

impl<V, T> Dynamic<V, T> {
    fn resolve(&self, ctx: RenderCtx, data: Arc<T>) -> BoxFuture<'static, anyhow::Result<V>> {
        (self.resolver)(ctx, data)
    }
}

/// Wraps an asynchronous callback as a dynamic property value.
///
/// This explicit adapter avoids ambiguity between ordinary static values and
/// callbacks while keeping the component property API uniform.
pub fn dynamic<V, T, F, Fut>(callback: F) -> Dynamic<V, T>
where
    F: Fn(RenderCtx, Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = anyhow::Result<V>> + Send + 'static,
    V: Send + 'static,
    T: Send + Sync + 'static,
{
    Dynamic {
        resolver: Arc::new(move |ctx, data| Box::pin(callback(ctx, data))),
    }
}

/// Converts an ordinary value or a dynamic callback into a property value.
pub trait IntoValue<V, T> {
    /// Converts this input into the framework's internal property representation.
    fn into_value(self) -> Value<V, T>;
}

impl<V, T> IntoValue<V, T> for Dynamic<V, T> {
    fn into_value(self) -> Value<V, T> {
        Value::Dynamic(self)
    }
}

impl<T> IntoValue<String, T> for String {
    fn into_value(self) -> Value<String, T> {
        Value::Static(self)
    }
}

impl<'a, T> IntoValue<String, T> for &'a str {
    fn into_value(self) -> Value<String, T> {
        Value::Static(self.to_owned())
    }
}

macro_rules! impl_static_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl<T> IntoValue<$ty, T> for $ty {
                fn into_value(self) -> Value<$ty, T> {
                    Value::Static(self)
                }
            }
        )+
    };
}

impl_static_value!(bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, f32, f64);

/// One fixed component slot, which may be empty or contain a static/dynamic value.
pub enum Slot<V, T> {
    /// No component occupies this slot.
    Empty,
    /// The slot contains a value to be resolved during rendering.
    Value(Value<V, T>),
}

impl<V: Clone, T> Clone for Slot<V, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Value(value) => Self::Value(value.clone()),
        }
    }
}

/// Converts ordinary optional values and dynamic callbacks into fixed slots.
pub trait IntoSlot<V, T> {
    /// Converts this input into either an empty slot or a populated slot.
    fn into_slot(self) -> Slot<V, T>;
}

impl<V, T> IntoSlot<V, T> for Dynamic<V, T> {
    fn into_slot(self) -> Slot<V, T> {
        Slot::Value(Value::Dynamic(self))
    }
}

impl<T> IntoSlot<String, T> for String {
    fn into_slot(self) -> Slot<String, T> {
        Slot::Value(Value::Static(self))
    }
}

impl<'a, T> IntoSlot<String, T> for &'a str {
    fn into_slot(self) -> Slot<String, T> {
        Slot::Value(Value::Static(self.to_owned()))
    }
}

impl<T> IntoSlot<String, T> for Option<String> {
    fn into_slot(self) -> Slot<String, T> {
        match self {
            Some(value) => Slot::Value(Value::Static(value)),
            None => Slot::Empty,
        }
    }
}

/// Resolves a property using the current Page data and App-linked context.
///
/// Static values are cloned. Dynamic callbacks are awaited and their original
/// errors are returned unchanged.
pub async fn resolve<V, T>(
    value: &Value<V, T>,
    ctx: RenderCtx,
    data: Arc<T>,
) -> anyhow::Result<V>
where
    V: Clone + Send + 'static,
    T: Send + Sync + 'static,
{
    match value {
        Value::Static(value) => Ok(value.clone()),
        Value::Dynamic(callback) => callback.resolve(ctx, data).await,
    }
}
