//! Container with ordered children, optional dynamic accent, spoiler, and Frames.
use crate::page::frame::Frame;
use crate::page::select::SelectMenu;
use crate::page::value::{IntoValue,Value};
use super::{Button,ButtonRow,ButtonSlot,ButtonSlots,Component,File,LinkButton,MediaGallery,
    Section,Separator,Text};

/// An ordered Container child, either a component or a dynamically expanded Frame.
pub enum ContainerItem<T> { Component(Component<T>), Frame(Frame<T>) }
/// Discord Container. Children remain in insertion order, including Frame entries.
pub struct Container<T> {
    /// Optional accent color resolved when the Page renders.
    pub accent_color:Option<Value<u32,T>>,
    /// Whether Discord marks the rendered Container as a spoiler.
    pub spoiler:bool,
    /// Ordered direct children; Discord permits at most ten.
    pub children:Vec<ContainerItem<T>>,
}
impl<T> Container<T> {
    /// Starts an empty, non-spoiler Container with no accent color.
    pub fn new()->Self {
        Self { accent_color:None,spoiler:false,children:Vec::new() }
    }
    /// Sets a static or dynamic accent color.
    pub fn set_accent_color<V:IntoValue<u32,T>>(mut self,value:V)->Self {
        self.accent_color=Some(value.into_value()); self
    }
    /// Sets the Container spoiler flag.
    pub fn set_spoiler(mut self,value:bool)->Self { self.spoiler=value; self }
    /// Appends a Text Display child.
    pub fn add_text<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.children.push(ContainerItem::Component(Component::Text(Text::new(value)))); self
    }
    /// Appends a Section child.
    pub fn add_section(mut self,value:Section<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Section(value))); self
    }
    /// Appends a Separator child.
    pub fn add_separator(mut self,value:Separator<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Separator(value))); self
    }
    /// Appends an interactive Button child.
    pub fn add_button(mut self,value:Button<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Button(value))); self
    }
    /// Appends a fixed-slot Button Action Row child.
    pub fn add_button_row<A, B, C, D, E>(
        mut self,
        slots: (A, B, C, D, E),
    ) -> Self
    where
        A: Into<ButtonSlot<T>>,
        B: Into<ButtonSlot<T>>,
        C: Into<ButtonSlot<T>>,
        D: Into<ButtonSlot<T>>,
        E: Into<ButtonSlot<T>>,
    {
        let slots = (
            slots.0.into(),
            slots.1.into(),
            slots.2.into(),
            slots.3.into(),
            slots.4.into(),
        );
        self.children.push(ContainerItem::Component(Component::ButtonRow(ButtonRow::new(slots))));
        self
    }
    /// Appends a URL Link Button child.
    pub fn add_link_button(mut self,value:LinkButton<T>)->Self {
        self.children.push(ContainerItem::Component(Component::LinkButton(value))); self
    }
    /// Appends a Select menu child; it occupies its own Action Row.
    pub fn add_selection_menu(mut self,value:SelectMenu<T>)->Self {
        self.children.push(ContainerItem::Component(Component::Select(value))); self
    }
    /// Appends a Media Gallery child.
    pub fn add_gallery(mut self,value:MediaGallery<T>)->Self {
        self.children.push(ContainerItem::Component(Component::MediaGallery(value))); self
    }
    /// Appends an attachment reference child.
    pub fn add_file(mut self,value:File)->Self {
        self.children.push(ContainerItem::Component(Component::File(value))); self
    }
    /// Appends a Frame that expands in place while preserving child order.
    pub fn add_frame(mut self,value:Frame<T>)->Self {
        self.children.push(ContainerItem::Frame(value)); self
    }
}
