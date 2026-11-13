//! Heterogeneous Frame output and stable developer-defined element keys.
use crate::page::component::*;
use crate::page::select::SelectMenu;

pub struct FrameIdentity {
    pub frame_id:String,
    pub element_key:String,
    pub execution_id:String,
}
pub struct Element<T> {
    pub component:Component<T>,
    pub key:Option<String>,
}
impl<T> Element<T> {
    pub fn text<V:crate::page::value::IntoValue<String,T>>(value:V)->Self {
        Self { component:Component::Text(Text::new(value)), key:None }
    }
    pub fn section(value:Section<T>)->Self {
        Self { component:Component::Section(value), key:None }
    }
    pub fn container(value:Container<T>)->Self {
        Self { component:Component::Container(value), key:None }
    }
    pub fn separator(value:Separator<T>)->Self {
        Self { component:Component::Separator(value), key:None }
    }
    pub fn button(value:Button<T>)->Self {
        Self { component:Component::Button(value), key:None }
    }
    pub fn button_row(value:ButtonRow<T>)->Self {
        Self { component:Component::ButtonRow(value), key:None }
    }
    pub fn link_button(value:LinkButton<T>)->Self {
        Self { component:Component::LinkButton(value), key:None }
    }
    pub fn gallery(value:MediaGallery<T>)->Self {
        Self { component:Component::MediaGallery(value), key:None }
    }
    pub fn file(value:File)->Self {
        Self { component:Component::File(value), key:None }
    }
    pub fn selection_menu(value:SelectMenu<T>)->Self {
        Self { component:Component::Select(value), key:None }
    }
    pub fn keyed(mut self,key:impl Into<String>)->Self {
        self.key=Some(key.into()); self
    }
}
