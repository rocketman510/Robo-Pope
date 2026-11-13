//! Media Gallery with dynamic per-item URLs/descriptions and spoiler settings.
use crate::page::value::{IntoValue,Value};

pub struct MediaItem<T,E> {
    pub url:Value<String,T,E>,
    pub description:Option<Value<String,T,E>>,
    pub spoiler:Value<bool,T,E>,
}
impl<T,E> MediaItem<T,E> {
    pub fn new<V:IntoValue<String,T,E>>(url:V)->Self {
        Self { url:url.into_value(), description:None, spoiler:Value::Static(false) }
    }
    pub fn set_description<V:IntoValue<String,T,E>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
    pub fn set_spoiler<V:IntoValue<bool,T,E>>(mut self,value:V)->Self {
        self.spoiler=value.into_value(); self
    }
}

pub struct MediaGallery<T,E> {
    pub items:Vec<MediaItem<T,E>>,
}
impl<T,E> MediaGallery<T,E> {
    pub fn new()->Self { Self { items:Vec::new() } }
    pub fn add_media<V:IntoValue<String,T,E>>(mut self,url:V)->Self {
        self.items.push(MediaItem::new(url)); self
    }
    pub fn add_item(mut self,item:MediaItem<T,E>)->Self {
        self.items.push(item); self
    }
}
