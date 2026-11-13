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

/// Ordered collection of up to ten Media Gallery items.
pub struct MediaGallery<T> {
    /// Items in display order. Discord permits at most ten.
    pub items:Vec<MediaItem<T>>,
}
impl<T> MediaGallery<T> {
    /// Starts an empty Media Gallery.
    pub fn new()->Self { Self { items:Vec::new() } }
    /// Appends a media item using the supplied URL.
    pub fn add_media<V:IntoValue<String,T>>(mut self,url:V)->Self {
        self.items.push(MediaItem::new(url)); self
    }
    /// Appends a fully configured item without changing its properties.
    pub fn add_item(mut self,item:MediaItem<T>)->Self {
        self.items.push(item); self
    }
}
