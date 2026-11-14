//! Media Gallery with dynamic per-item URLs/descriptions and spoiler settings.
use crate::page::value::{IntoValue,Value};

/// One Media Gallery item. URL, description, and spoiler state resolve per render.
pub struct MediaItem<T> {
    /// Media URL resolved at render time.
    pub url:Value<String,T>,
    /// Optional media description resolved at render time.
    pub description:Option<Value<String,T>>,
    /// Whether Discord marks this media item as a spoiler.
    pub spoiler:Value<bool,T>,
}
impl<T> MediaItem<T> {
    /// Creates an item with a URL and default non-spoiler state.
    pub fn new<V:IntoValue<String,T>>(url:V)->Self {
        Self { url:url.into_value(), description:None, spoiler:Value::Static(false) }
    }
    /// Sets an optional static or dynamic description.
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    /// Sets the item's static or dynamic spoiler state.
    pub fn set_spoiler<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.spoiler=value.into_value(); self
    }
}

/// Converts either a fully configured item or a URL value into a Media Item.
pub trait IntoMediaItem<T> {
    fn into_media_item(self) -> MediaItem<T>;
}

impl<T> IntoMediaItem<T> for MediaItem<T> {
    fn into_media_item(self) -> MediaItem<T> { self }
}

impl<T> IntoMediaItem<T> for String {
    fn into_media_item(self) -> MediaItem<T> { MediaItem::new(self) }
}

impl<'a, T> IntoMediaItem<T> for &'a str {
    fn into_media_item(self) -> MediaItem<T> { MediaItem::new(self) }
}

impl<T> IntoMediaItem<T> for crate::page::value::Value<String, T> {
    fn into_media_item(self) -> MediaItem<T> { MediaItem { url: self, description: None, spoiler: crate::page::value::Value::Static(false) } }
}

impl<T, F, Fut> IntoMediaItem<T> for F
where
    F: Fn(crate::app::RenderCtx, std::sync::Arc<T>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = anyhow::Result<String>> + Send + 'static,
    T: Send + Sync + 'static,
{
    fn into_media_item(self) -> MediaItem<T> { MediaItem::new(self) }
}

/// Ordered collection of up to ten Media Gallery items.
pub struct MediaGallery<T> {
    /// Items in display order. Discord permits at most ten.
    pub items:Vec<MediaItem<T>>,
}
impl<T> MediaGallery<T> {
    /// Starts an empty Media Gallery.
    pub fn new()->Self { Self { items:Vec::new() } }
    /// Replaces the gallery's current collection of media items.
    pub fn set_media<I, M>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = M>,
        M: IntoMediaItem<T>,
    {
        self.items = items.into_iter().map(IntoMediaItem::into_media_item).collect();
        self
    }
    /// Appends a URL or a fully configured Media Item.
    pub fn add_media<M: IntoMediaItem<T>>(mut self, item: M) -> Self {
        self.items.push(item.into_media_item());
        self
    }
    /// Appends a fully configured item without changing its properties.
    pub fn add_item(mut self, item: MediaItem<T>) -> Self {
        self.items.push(item);
        self
    }
}
