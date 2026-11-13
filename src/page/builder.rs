//! Page<T, E> builder and immutable RegisteredPage<T, E> typestate.
use std::sync::Arc;
use serde::{Serialize, de::DeserializeOwned};
use crate::app::App;
use super::component::*;
use super::frame::Frame;
use super::render::{RenderedMessage, RenderedPage};
use super::select::SelectMenu;
use super::value::IntoValue;

pub struct Page<T, E = ()> {
    pub(crate) id: String,
    pub(crate) items: Vec<PageItem<T, E>>,
    pub(crate) paginate: bool,
}
pub struct PageDefinition<T, E> {
    pub(crate) id: String,
    pub(crate) items: Vec<PageItem<T, E>>,
    pub(crate) paginate: bool,
}
pub enum PageItem<T, E> {
    Component(Component<T, E>),
    Frame(Frame<T, E>),
}
pub struct RegisteredPage<T, E = ()> {
    pub(crate) definition: Arc<PageDefinition<T, E>>,
}

impl<T, E> Page<T, E> {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id:id.into(), items:Vec::new(), paginate:false }
    }
    pub fn id(&self) -> &str { &self.id }
    pub fn add_text<V: IntoValue<String, T, E>>(mut self, value: V) -> Self {
        self.items.push(PageItem::Component(Component::Text(Text::new(value))));
        self
    }
    pub fn add_section(mut self, value: Section<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::Section(value))); self
    }
    pub fn add_container(mut self, value: Container<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::Container(value))); self
    }
    pub fn add_separator(mut self, value: Separator<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::Separator(value))); self
    }
    pub fn add_button(mut self, value: Button<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::Button(value))); self
    }
    pub fn add_link_button(mut self, value: LinkButton<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::LinkButton(value))); self
    }
    pub fn add_button_row(mut self, slots: ButtonSlots<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::ButtonRow(ButtonRow::new(slots))));
        self
    }
    pub fn add_selection_menu<M: Into<SelectMenu<T, E>>>(mut self, value: M) -> Self {
        self.items.push(PageItem::Component(Component::Select(value.into()))); self
    }
    pub fn add_gallery(mut self, value: MediaGallery<T, E>) -> Self {
        self.items.push(PageItem::Component(Component::MediaGallery(value))); self
    }
    pub fn add_file(mut self, value: File) -> Self {
        self.items.push(PageItem::Component(Component::File(value))); self
    }
    pub fn add_frame(mut self, value: Frame<T, E>) -> Self {
        self.items.push(PageItem::Frame(value)); self
    }
    pub fn paginate(mut self) -> Self { self.paginate = true; self }

    pub fn register(self, app: &App<E>) -> anyhow::Result<RegisteredPage<T, E>>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
        E: Send + Sync + 'static,
    {
        todo!("consume builder, validate static invariants, and register immutable definition")
    }
}

impl<T, E> RegisteredPage<T, E>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    pub fn id(&self) -> &str { &self.definition.id }

    pub async fn render(&self, app: &App<E>, data: T)
        -> anyhow::Result<RenderedMessage>
    {
        todo!("resolve values and Frames, validate, paginate, persist executions, build message")
    }

    pub async fn render_pages(&self, app: &App<E>, data: T)
        -> anyhow::Result<RenderedPage>
    {
        todo!("render all Frame pages and expose pagination metadata")
    }
}
