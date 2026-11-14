//! Heterogeneous components emitted by a dynamic Frame.
use std::sync::Arc;
use crate::page::component::*;
use crate::page::select::SelectMenu;

/// One heterogeneous component emitted by a Frame.
pub struct Element<T, D = Arc<()>> {
    pub component: Component<T, D>,
}
impl<T, D> Element<T, D> {
    pub fn text<V: crate::page::value::IntoValue<String, T, D>>(value: V) -> Self {
        Self { component: Component::Text(Text::new(value)) }
    }
    pub fn section(value: Section<T, D>) -> Self {
        Self { component: Component::Section(value) }
    }
    pub fn container(value: Container<T, D>) -> Self {
        Self { component: Component::Container(value) }
    }
    pub fn separator(value: Separator<T, D>) -> Self {
        Self { component: Component::Separator(value) }
    }
    pub fn button(value: Button<T, D>) -> Self {
        Self { component: Component::Button(value) }
    }
    pub fn button_row(value: ButtonRow<T, D>) -> Self {
        Self { component: Component::ButtonRow(value) }
    }
    pub fn link_button(value: LinkButton<T, D>) -> Self {
        Self { component: Component::LinkButton(value) }
    }
    pub fn gallery(value: MediaGallery<T, D>) -> Self {
        Self { component: Component::MediaGallery(value) }
    }
    pub fn file(value: File) -> Self {
        Self { component: Component::File(value) }
    }
    pub fn selection_menu(value: SelectMenu<T, D>) -> Self {
        Self { component: Component::Select(value) }
    }
}
