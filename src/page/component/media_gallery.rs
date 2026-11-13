//! Media Gallery with dynamic per-item URLs/descriptions and spoiler settings.
use crate::page::value::{IntoValue,Value};

pub struct MediaItem<T> {
    pub url:Value<String,T>,
    pub description:Option<Value<String,T>>,
    pub spoiler:Value<bool,T>,
}
impl<T> MediaItem<T> {
    pub fn new<V:IntoValue<String,T>>(url:V)->Self {
        Self { url:url.into_value(), description:None, spoiler:Value::Static(false) }
    }
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    pub fn set_spoiler<V:IntoValue<bool,T>>(mut self,value:V)->Self {
        self.spoiler=value.into_value(); self
    }
}

pub struct MediaGallery<T> {
    pub items:Vec<MediaItem<T>>,
}
impl<T> MediaGallery<T> {
    pub fn new()->Self { Self { items:Vec::new() } }
    pub fn add_media<V:IntoValue<String,T>>(mut self,url:V)->Self {
        self.items.push(MediaItem::new(url)); self
    }
    pub fn add_item(mut self,item:MediaItem<T>)->Self {
        self.items.push(item); self
    }
}
