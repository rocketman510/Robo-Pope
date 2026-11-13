//! Heterogeneous Frame output and stable developer-defined element keys.
use crate::page::component::*;
use crate::page::select::SelectMenu;

pub struct FrameIdentity {
    pub frame_id:String,
    pub element_key:String,
    pub execution_id:String,
}
pub struct Element<T,E> {
    pub component:Component<T,E>,
    pub key:Option<String>,
}
impl<T,E> Element<T,E> {
    pub fn text<V:crate::page::value::IntoValue<String,T,E>>(value:V)->Self {
        Self { component:Component::Text(Text::new(value)), key:None }
    }
    pub fn section(value:Section<T,E>)->Self {
        Self { component:Component::Section(value), key:None }
    }
    pub fn container(value:Container<T,E>)->Self {
        Self { component:Component::Container(value), key:None }
    }
    pub fn separator(value:Separator<T,E>)->Self {
        Self { component:Component::Separator(value), key:None }
    }
    pub fn button(value:Button<T,E>)->Self {
        Self { component:Component::Button(value), key:None }
    }
    pub fn button_row(value:ButtonRow<T,E>)->Self {
        Self { component:Component::ButtonRow(value), key:None }
    }
    pub fn link_button(value:LinkButton<T,E>)->Self {
        Self { component:Component::LinkButton(value), key:None }
    }
    pub fn gallery(value:MediaGallery<T,E>)->Self {
        Self { component:Component::MediaGallery(value), key:None }
    }
    pub fn file(value:File)->Self {
        Self { component:Component::File(value), key:None }
    }
    pub fn selection_menu(value:SelectMenu<T,E>)->Self {
        Self { component:Component::Select(value), key:None }
    }
    pub fn keyed(mut self,key:impl Into<String>)->Self {
        self.key=Some(key.into()); self
    }
}
