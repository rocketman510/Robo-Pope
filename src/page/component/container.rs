//! Container with ordered children, optional dynamic accent, spoiler, and Frames.
use crate::page::frame::Frame;
use crate::page::select::SelectMenu;
use crate::page::value::{IntoValue,Value};
use super::{Button,ButtonRow,ButtonSlots,Component,File,LinkButton,MediaGallery,
    Section,Separator,Text};

pub enum ContainerItem<T> { Component(Component<T>), Frame(Frame<T>) }
pub struct Container<T> {
    pub accent_color:Option<Value<u32,T>>,
    pub spoiler:bool,
    pub children:Vec<ContainerItem<T>>,
}
impl<T> Container<T> {
    pub fn new()->Self {
        Self { accent_color:None,spoiler:false,children:Vec::new() }
    }
    pub fn set_accent_color<V:IntoValue<u32,T>>(mut self,value:V)->Self {
        self.accent_color=Some(value.into_value()); self
    }
    pub fn set_spoiler(mut self,value:bool)->Self { self.spoiler=value; self }
    pub fn add_text<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.children.push(ContainerItem::Component(Component::Text(Text::new(value)))); self
    }
    pub fn add_section(mut self,value:Section<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Section(value))); self
    }
    pub fn add_separator(mut self,value:Separator<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Separator(value))); self
    }
    pub fn add_button(mut self,value:Button<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Button(value))); self
    }
    pub fn add_button_row(mut self,slots:ButtonSlots<T>)->Self {
        self.children.push(ContainerItem::Component(Component::ButtonRow(ButtonRow::new(slots)))); self
    }
    pub fn add_link_button(mut self,value:LinkButton<T>)->Self {
        self.children.push(ContainerItem::Component(Component::LinkButton(value))); self
    }
    pub fn add_selection_menu(mut self,value:SelectMenu<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Select(value))); self
    }
    pub fn add_gallery(mut self,value:MediaGallery<T>)->Self {
        self.children.push(ContainerItem::Component(Component::MediaGallery(value))); self
    }
    pub fn add_file(mut self,value:File)->Self {
        self.children.push(ContainerItem::Component(Component::File(value))); self
    }
    pub fn add_frame(mut self,value:Frame<T>)->Self {
        self.children.push(ContainerItem::Frame(value)); self
    }
}
