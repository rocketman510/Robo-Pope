//! Page<T> builder and immutable RegisteredPage<T> typestate.
use std::sync::Arc;
use std::collections::HashSet;
use serde::{Serialize, de::DeserializeOwned};
use crate::app::App;
use super::component::*;
use super::frame::Frame;
use super::render::{RenderedMessage, RenderedPage};
use super::select::SelectMenu;
use super::value::IntoValue;

/// Mutable builder for a reusable message definition.
///
/// `T` is the Page's shared data type. It is serialized for each rendered
/// instance and reconstructed before an interaction callback runs.
pub struct Page<T> {
    pub(crate) id: String,
    pub(crate) items: Vec<PageItem<T>>,
    pub(crate) paginate: bool,
}
/// Immutable definition retained by the App after Page registration.
pub struct PageDefinition<T> {
    pub(crate) id: String,
    pub(crate) items: Vec<PageItem<T>>,
    pub(crate) paginate: bool,
}
/// One ordered top-level entry in a Page: a fixed component or dynamic Frame.
pub enum PageItem<T> {
    Component(Component<T>),
    Frame(Frame<T>),
}
/// Registered, immutable Page handle. Rendering is only available after registration.
pub struct RegisteredPage<T> {
    pub(crate) definition: Arc<PageDefinition<T>>,
}


fn validate_execution_id(id: &str, ids: &mut HashSet<String>) -> anyhow::Result<()> {
    anyhow::ensure!(!id.trim().is_empty(), "Execution ID must not be empty");
    anyhow::ensure!(ids.insert(id.to_owned()), "Duplicate execution ID: {id}");
    Ok(())
}

fn validate_frame_id(id: &str, ids: &mut HashSet<String>) -> anyhow::Result<()> {
    anyhow::ensure!(!id.trim().is_empty(), "Frame ID must not be empty");
    anyhow::ensure!(ids.insert(id.to_owned()), "Duplicate Frame ID: {id}");
    Ok(())
}

fn validate_component_ids<T>(
    component: &Component<T>,
    execution_ids: &mut HashSet<String>,
    frame_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    use super::component::{ContainerItem, SectionAccessory};
    use super::select::SelectMenu;

    match component {
        Component::Button(button) => {
            validate_execution_id(&button.id, execution_ids)?;
        }
        Component::ButtonRow(row) => {
            for button in [
                row.slots.0.as_ref(),
                row.slots.1.as_ref(),
                row.slots.2.as_ref(),
                row.slots.3.as_ref(),
                row.slots.4.as_ref(),
            ].into_iter().flatten() {
                validate_execution_id(&button.id, execution_ids)?;
            }
        }
        Component::Section(section) => {
            if let Some(SectionAccessory::Button(button)) = &section.accessory {
                validate_execution_id(&button.id, execution_ids)?;
            }
        }
        Component::Select(menu) => {
            let id = match menu {
                SelectMenu::String(menu) => &menu.id,
                SelectMenu::User(menu) => &menu.id,
                SelectMenu::Role(menu) => &menu.id,
                SelectMenu::Mentionable(menu) => &menu.id,
                SelectMenu::Channel(menu) => &menu.id,
            };
            validate_execution_id(id, execution_ids)?;
        }
        Component::Container(container) => {
            for child in &container.children {
                match child {
                    ContainerItem::Component(component) => {
                        validate_component_ids(component, execution_ids, frame_ids)?;
                    }
                    ContainerItem::Frame(frame) => {
                        validate_frame_id(&frame.id, frame_ids)?;
                    }
                }
            }
        }
        Component::Text(_)
        | Component::Separator(_)
        | Component::MediaGallery(_)
        | Component::File(_)
        | Component::LinkButton(_) => {}
    }
    Ok(())
}

impl<T> Page<T> {
    /// Starts a Page definition with a globally unique application-wide ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id:id.into(), items:Vec::new(), paginate:false }
    }
    /// Returns the developer-defined Page ID.
    pub fn id(&self) -> &str { &self.id }
    /// Appends a Text Display. Its content may be static or resolved at render time.
    pub fn add_text<V: IntoValue<String, T>>(mut self, value: V) -> Self {
        self.items.push(PageItem::Component(Component::Text(Text::new(value))));
        self
    }
    /// Appends a Section with one to three Text Display slots and one optional accessory.
    pub fn add_section(mut self, value: Section<T>) -> Self {
        self.items.push(PageItem::Component(Component::Section(value))); self
    }
    /// Appends a Container while preserving the order of its children.
    pub fn add_container(mut self, value: Container<T>) -> Self {
        self.items.push(PageItem::Component(Component::Container(value))); self
    }
    /// Appends a Separator with dynamic spacing and divider properties.
    pub fn add_separator(mut self, value: Separator<T>) -> Self {
        self.items.push(PageItem::Component(Component::Separator(value))); self
    }
    /// Appends an interactive Button with an explicit execution callback.
    pub fn add_button(mut self, value: Button<T>) -> Self {
        self.items.push(PageItem::Component(Component::Button(value))); self
    }
    /// Appends a URL button. Link buttons do not create interaction executions.
    pub fn add_link_button(mut self, value: LinkButton<T>) -> Self {
        self.items.push(PageItem::Component(Component::LinkButton(value))); self
    }
    /// Appends an Action Row with up to five fixed Button slots.
    pub fn add_button_row(mut self, slots: ButtonSlots<T>) -> Self {
        self.items.push(PageItem::Component(Component::ButtonRow(ButtonRow::new(slots))));
        self
    }
    /// Appends one String, User, Role, Mentionable, or Channel Select menu.
    pub fn add_selection_menu<M: Into<SelectMenu<T>>>(mut self, value: M) -> Self {
        self.items.push(PageItem::Component(Component::Select(value.into()))); self
    }
    /// Appends a Media Gallery containing at most ten media items.
    pub fn add_gallery(mut self, value: MediaGallery<T>) -> Self {
        self.items.push(PageItem::Component(Component::MediaGallery(value))); self
    }
    /// Appends a reference to an attachment included in the outgoing message.
    pub fn add_file(mut self, value: File) -> Self {
        self.items.push(PageItem::Component(Component::File(value))); self
    }
    /// Appends a dynamic region whose elements are resolved when the Page renders.
    pub fn add_frame(mut self, value: Frame<T>) -> Self {
        self.items.push(PageItem::Frame(value)); self
    }
    /// Enables framework-managed pagination for dynamic Frame content.
    pub fn paginate(mut self) -> Self { self.paginate = true; self }

    /// Validates and registers this definition. Registration consumes the builder;
    /// a duplicate Page ID is an error.
    pub fn register(self, app: &App) -> anyhow::Result<RegisteredPage<T>>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static
{
        anyhow::ensure!(!self.id.trim().is_empty(), "Page ID must not be empty");
        let mut execution_ids = HashSet::new();
        let mut frame_ids = HashSet::new();
        for item in &self.items {
            match item {
                PageItem::Component(component) => {
                    super::render::validate_component(component)?;
                    validate_component_ids(component, &mut execution_ids, &mut frame_ids)?;
                }
                PageItem::Frame(frame) => {
                    validate_frame_id(&frame.id, &mut frame_ids)?;
                }
            }
        }
        let definition = Arc::new(PageDefinition {
            id: self.id,
            items: self.items,
            paginate: self.paginate,
        });
        app.pages.insert(definition.clone())?;
        Ok(RegisteredPage { definition })
    }
}

impl<T> RegisteredPage<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static
{
    /// Returns the registered Page ID.
    pub fn id(&self) -> &str { &self.definition.id }

    /// Resolves dynamic properties and Frames for one Page instance, persists
    /// execution references, checks Discord limits, and returns one message.
    pub async fn render(&self, app: &App, data: T)
        -> anyhow::Result<RenderedMessage>
    {
        todo!("resolve values and Frames, validate, paginate, persist executions, build message")
    }

    /// Renders the Page's complete Frame pagination and reports its current position.
    pub async fn render_pages(&self, app: &App, data: T)
        -> anyhow::Result<RenderedPage>
    {
        todo!("render all Frame pages and expose pagination metadata")
    }
}
