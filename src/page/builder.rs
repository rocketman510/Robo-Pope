//! Page<T> builder and immutable RegisteredPage<T, D> typestate.
use std::sync::Arc;
use std::future::Future;
use std::collections::HashSet;
use anyhow::Context as _;
use serenity::builder::{
    CreateActionRow, CreateButton, CreateComponent, CreateContainer,
    CreateContainerComponent, CreateFile, CreateMediaGallery,
    CreateMediaGalleryItem, CreateSection, CreateSectionAccessory,
    CreateSectionComponent, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, CreateSeparator, CreateTextDisplay,
    CreateThumbnail, CreateUnfurledMediaItem,
};
use serenity::all::ButtonStyle as SerenityButtonStyle;
use serde::{Serialize, de::DeserializeOwned};
use crate::app::{App, RenderCtx};
use crate::app::store::{derive_custom_id, RoutingRecord};
use super::component::*;
use super::frame::Frame;
use super::render::{RenderedComponent, RenderedMessage};
use super::select::SelectMenu;
use super::value::IntoValue;

/// Mutable builder for a reusable message definition.
///
/// `T` is the Page's shared data type. It is serialized for each rendered
/// instance and reconstructed before an interaction callback runs.
pub struct Page<T, D = Arc<()>> {
    pub(crate) id: String,
    pub(crate) items: Vec<PageItem<T, D>>,
}
/// Immutable definition retained by the App after Page registration.
pub struct PageDefinition<T, D = Arc<()>> {
    pub(crate) id: String,
    pub(crate) items: Vec<PageItem<T, D>>,
}
/// One ordered top-level entry in a Page: a fixed component or dynamic Frame.
pub enum PageItem<T, D = Arc<()>> {
    Component(Component<T, D>),
    Frame(Frame<T, D>),
}
/// Registered, immutable Page handle. Rendering is only available after registration.
pub struct RegisteredPage<T, D = Arc<()>> {
    pub(crate) definition: Arc<PageDefinition<T, D>>,
}


fn validate_execution_id(id: &str, ids: &mut HashSet<String>) -> anyhow::Result<()> {
    anyhow::ensure!(!id.trim().is_empty(), "Execution ID must not be empty");
    anyhow::ensure!(ids.insert(id.to_owned()), "Duplicate execution ID: {id}");
    Ok(())
}

fn validate_component_ids<T, D>(
    component: &Component<T, D>,
    execution_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    use super::component::{ButtonSlot, ContainerItem, SectionAccessory};
    use super::select::SelectMenu;
    match component {
        Component::Button(button) => validate_execution_id(&button.id, execution_ids)?,
        Component::ButtonRow(row) => {
            for slot in [&row.slots.0, &row.slots.1, &row.slots.2, &row.slots.3, &row.slots.4] {
                if let ButtonSlot::Button(button) = slot {
                    validate_execution_id(&button.id, execution_ids)?;
                }
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
                if let ContainerItem::Component(component) = child {
                    validate_component_ids(component, execution_ids)?;
                }
            }
        }
        Component::Text(_) | Component::Separator(_) | Component::MediaGallery(_)
        | Component::File(_) | Component::LinkButton(_) => {}
    }
    Ok(())
}

fn static_component_count<T, D>(component: &Component<T, D>) -> usize {
    use super::component::{Component as C, ContainerItem};
    use super::value::Slot;

    match component {
        C::Button(_) | C::LinkButton(_) | C::Select(_) => 2,
        C::ButtonRow(row) => 1 + row.populated(),
        C::Section(section) => {
            let text_count = [&section.text.0, &section.text.1, &section.text.2]
                .iter()
                .filter(|slot| matches!(slot, Slot::Value(_)))
                .count();
            1 + text_count + if section.accessory.is_some() { 1 } else { 0 }
        }
        C::Container(container) => {
            1 + container.children.iter().map(|child| match child {
                ContainerItem::Component(component) => static_component_count(component),
                ContainerItem::Frame(_) => 0,
            }).sum::<usize>()
        }
        C::Text(_) | C::Separator(_) | C::MediaGallery(_) | C::File(_) => 1,
    }
}

fn register_component_executions<T, D>(
    component: &Component<T, D>,
    app: &App<D>,
    definition_id: &str,
) -> anyhow::Result<()>
where
    T: DeserializeOwned + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    use super::component::{ContainerItem, SectionAccessory};
    match component {
        Component::Button(button) => button.execution.register(app, definition_id)?,
        Component::ButtonRow(row) => {
            for slot in [&row.slots.0, &row.slots.1, &row.slots.2, &row.slots.3, &row.slots.4] {
                if let ButtonSlot::Button(button) = slot {
                    button.execution.register(app, definition_id)?;
                }
            }
        }
        Component::Section(section) => {
            if let Some(SectionAccessory::Button(button)) = &section.accessory {
                button.execution.register(app, definition_id)?;
            }
        }
        Component::Select(menu) => match menu {
            SelectMenu::String(menu) => menu.execution.register(app, definition_id)?,
            SelectMenu::User(menu) => menu.execution.register(app, definition_id)?,
            SelectMenu::Role(menu) => menu.execution.register(app, definition_id)?,
            SelectMenu::Mentionable(menu) => menu.execution.register(app, definition_id)?,
            SelectMenu::Channel(menu) => menu.execution.register(app, definition_id)?,
        },
        Component::Container(container) => {
            for child in &container.children {
                if let ContainerItem::Component(component) = child {
                    register_component_executions(component, app, definition_id)?;
                }
            }
        }
        Component::Text(_) | Component::Separator(_) | Component::MediaGallery(_)
        | Component::File(_) | Component::LinkButton(_) => {}
    }
    Ok(())
}

impl<T, D> Page<T, D> {
    /// Starts a Page definition with a globally unique application-wide ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id:id.into(), items:Vec::new() }
    }
    /// Returns the developer-defined Page ID.
    pub fn id(&self) -> &str { &self.id }
    /// Appends a Text Display. Its content may be static or resolved at render time.
    pub fn add_text<V: IntoValue<String, T, D>>(mut self, value: V) -> Self {
        self.items.push(PageItem::Component(Component::Text(Text::new(value))));
        self
    }
    /// Appends a Section with one to three Text Display slots and one optional accessory.
    pub fn add_section(mut self, value: Section<T, D>) -> Self {
        self.items.push(PageItem::Component(Component::Section(value))); self
    }
    /// Appends a Container while preserving the order of its children.
    pub fn add_container(mut self, value: Container<T, D>) -> Self {
        self.items.push(PageItem::Component(Component::Container(value))); self
    }
    /// Appends a Separator with dynamic spacing and divider properties.
    pub fn add_separator(mut self, value: Separator<T, D>) -> Self {
        self.items.push(PageItem::Component(Component::Separator(value))); self
    }
    /// Appends an interactive Button with an explicit execution callback.
    pub fn add_button(mut self, value: Button<T, D>) -> Self {
        self.items.push(PageItem::Component(Component::Button(value))); self
    }
    /// Appends a URL button. Link buttons do not create interaction executions.
    pub fn add_link_button(mut self, value: LinkButton<T, D>) -> Self {
        self.items.push(PageItem::Component(Component::LinkButton(value))); self
    }
    /// Appends an Action Row with up to five fixed Button slots.
    pub fn add_button_row<A, B, C, SlotD, SlotE>(mut self, slots: (A, B, C, SlotD, SlotE)) -> Self
    where
        A: Into<ButtonSlot<T, D>>,
        B: Into<ButtonSlot<T, D>>,
        C: Into<ButtonSlot<T, D>>,
        SlotD: Into<ButtonSlot<T, D>>,
        SlotE: Into<ButtonSlot<T, D>>,
    {
        let slots = (
            slots.0.into(),
            slots.1.into(),
            slots.2.into(),
            slots.3.into(),
            slots.4.into(),
        );
        self.items.push(PageItem::Component(Component::ButtonRow(ButtonRow::new(slots))));
        self
    }
    /// Appends one String, User, Role, Mentionable, or Channel Select menu.
    pub fn add_selection_menu<M: Into<SelectMenu<T, D>>>(mut self, value: M) -> Self {
        self.items.push(PageItem::Component(Component::Select(value.into()))); self
    }
    /// Appends a Media Gallery containing at most ten media items.
    pub fn add_gallery(mut self, value: MediaGallery<T, D>) -> Self {
        self.items.push(PageItem::Component(Component::MediaGallery(value))); self
    }
    /// Appends a reference to an attachment included in the outgoing message.
    pub fn add_file(mut self, value: File) -> Self {
        self.items.push(PageItem::Component(Component::File(value))); self
    }
    /// Appends a dynamic region whose elements are resolved when the Page renders.
    pub fn add_frame<F, Fut>(mut self, resolver: F) -> Self
    where
        F: Fn(RenderCtx<D>, Arc<T>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = anyhow::Result<Vec<super::frame::Element<T, D>>>> + Send + 'static,
        T: Send + Sync + 'static,
        D: Clone + Send + Sync + 'static,
    {
        self.items.push(PageItem::Frame(Frame::new(resolver)));
        self
    }
    /// Validates and registers this definition. Registration consumes the builder;
    /// a duplicate Page ID is an error.
    pub fn register(self, app: &App<D>) -> anyhow::Result<RegisteredPage<T, D>>
    where
        D: Clone + Send + Sync + 'static,
        T: Serialize + DeserializeOwned + Send + Sync + 'static
    {
        anyhow::ensure!(!self.id.trim().is_empty(), "Page ID must not be empty");
        let static_count = self.items.iter().map(|item| match item {
            PageItem::Component(component) => static_component_count(component),
            PageItem::Frame(_) => 0,
        }).sum::<usize>();
        anyhow::ensure!(
            static_count <= super::limits::MESSAGE_MAX_COMPONENTS,
            "Page '{}' contains {} static message components; maximum is {}",
            self.id,
            static_count,
            super::limits::MESSAGE_MAX_COMPONENTS
        );
        let mut execution_ids = HashSet::new();
        for item in &self.items {
            if let PageItem::Component(component) = item {
                super::render::validate_component(component)?;
                validate_component_ids(component, &mut execution_ids)?;
            }
        }
        app.reserve_definition_id(&self.id)?;
        for item in &self.items {
            if let PageItem::Component(component) = item {
                if let Err(error) = register_component_executions(component, &app, &self.id) {
                    app.executions.remove_definition(&self.id);
                    app.release_definition_id(&self.id);
                    return Err(error);
                }
            }
        }
        let definition = Arc::new(PageDefinition {
            id: self.id,
            items: self.items,
        });
        if let Err(error) = app.pages.insert(definition.clone()) {
            app.executions.remove_definition(&definition.id);
            app.release_definition_id(&definition.id);
            return Err(error);
        }
        Ok(RegisteredPage { definition })
    }
}


#[derive(Debug, Clone, Copy, Default)]
struct ComponentBudget {
    total_components: usize,
    text_display_chars: usize,
    direct_container_children: usize,
}

fn add_budgets(left: ComponentBudget, right: ComponentBudget) -> ComponentBudget {
    ComponentBudget {
        total_components: left.total_components.saturating_add(right.total_components),
        text_display_chars: left.text_display_chars.saturating_add(right.text_display_chars),
        direct_container_children: left.direct_container_children.max(right.direct_container_children),
    }
}

fn validate_message_budget(budget: ComponentBudget) -> anyhow::Result<()> {
    anyhow::ensure!(
        budget.total_components <= super::limits::MESSAGE_MAX_COMPONENTS,
        "Rendered message has {} components; maximum is {}",
        budget.total_components,
        super::limits::MESSAGE_MAX_COMPONENTS
    );
    anyhow::ensure!(
        budget.text_display_chars <= super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS,
        "Rendered message has {} Text Display characters; maximum is {}",
        budget.text_display_chars,
        super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS
    );
    Ok(())
}

fn persist_button_execution<T, D>(
    button: &Button<T, D>,
    definition_id: &str,
    serialized_data: &[u8],
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<String> {
    let custom_id = derive_custom_id(definition_id, &button.execution.id, serialized_data)?;
    routes.push(RoutingRecord {
        hash: custom_id.0.clone(),
        definition_id: definition_id.to_owned(),
        execution_id: button.execution.id.clone(),
        serialized_data: serialized_data.to_vec(),
    });
    Ok(custom_id.0)
}

fn persist_select_execution(
    execution_id: &str,
    definition_id: &str,
    serialized_data: &[u8],
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<String> {
    let custom_id = derive_custom_id(definition_id, execution_id, serialized_data)?;
    routes.push(RoutingRecord {
        hash: custom_id.0.clone(),
        definition_id: definition_id.to_owned(),
        execution_id: execution_id.to_owned(),
        serialized_data: serialized_data.to_vec(),
    });
    Ok(custom_id.0)
}

async fn render_link_button<T, D>(
    button: &LinkButton<T, D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
) -> anyhow::Result<CreateButton<'static>>
where
    T: Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    let label = super::value::resolve(&button.label, ctx.clone(), data.clone())
        .await.context("Resolving Link Button label")?;
    let url = super::value::resolve(&button.url, ctx.clone(), data.clone())
        .await.context("Resolving Link Button URL")?;
    let emoji = super::value::resolve(&button.emoji, ctx.clone(), data.clone())
        .await.context("Resolving Link Button emoji")?;
    let disabled = super::value::resolve(&button.disabled, ctx.clone(), data.clone())
        .await.context("Resolving Link Button disabled state")?;

    let mut rendered = CreateButton::new_link(url).label(label).disabled(disabled);
    if let Some(emoji) = emoji {
        rendered = rendered.emoji(serenity::all::ReactionType::try_from(emoji.as_str())?);
    }
    Ok(rendered.into_owned())
}

async fn render_button<T, D>(
    button: &Button<T, D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
    definition_id: &str,
    serialized_data: &[u8],
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<CreateButton<'static>>
where
    T: Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    let label = super::value::resolve(&button.label, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving label for Button '{}'", button.id))?;
    let style = super::value::resolve(&button.style, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving style for Button '{}'", button.id))?;
    let emoji = super::value::resolve(&button.emoji, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving emoji for Button '{}'", button.id))?;
    let disabled = super::value::resolve(&button.disabled, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving disabled state for Button '{}'", button.id))?;
    let custom_id = persist_button_execution(
        button, definition_id, serialized_data, routes,
    )?;

    let serenity_style = match style {
        super::component::ButtonStyle::Primary => SerenityButtonStyle::Primary,
        super::component::ButtonStyle::Secondary => SerenityButtonStyle::Secondary,
        super::component::ButtonStyle::Success => SerenityButtonStyle::Success,
        super::component::ButtonStyle::Danger => SerenityButtonStyle::Danger,
    };

    let mut rendered = CreateButton::new(custom_id)
        .label(label)
        .style(serenity_style)
        .disabled(disabled);
    if let Some(emoji) = emoji {
        rendered = rendered.emoji(serenity::all::ReactionType::try_from(emoji.as_str())?);
    }
    Ok(rendered.into_owned())
}

async fn render_select_menu<T, D>(
    menu: &SelectMenu<T, D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<CreateSelectMenu<'static>>
where
    T: Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    use super::select::SelectMenu as M;

    let (execution_id, placeholder, disabled, min_values, max_values, kind) = match menu {
        M::String(menu) => {
            let mut options = Vec::with_capacity(menu.options.len());
            for option in &menu.options {
                let label = super::value::resolve(&option.label, ctx.clone(), data.clone())
                    .await.context("Resolving String Select option label")?;
                anyhow::ensure!(
                    label.chars().count() <= super::limits::SELECT_OPTION_MAX_CHARS,
                    "String Select option label exceeds {} characters",
                    super::limits::SELECT_OPTION_MAX_CHARS
                );
                anyhow::ensure!(
                    option.value.chars().count() <= super::limits::SELECT_OPTION_MAX_CHARS,
                    "String Select option value exceeds {} characters",
                    super::limits::SELECT_OPTION_MAX_CHARS
                );
                let disabled = super::value::resolve(
                    &option.disabled, ctx.clone(), data.clone()
                ).await.context("Resolving String Select option disabled state")?;
                anyhow::ensure!(
                    !disabled,
                    "Discord String Select options do not support per-option disabled state"
                );
                let mut rendered = CreateSelectMenuOption::new(label, option.value.clone());
                if let Some(description) = &option.description {
                    let description = super::value::resolve(
                        description, ctx.clone(), data.clone()
                    ).await.context("Resolving String Select option description")?;
                    anyhow::ensure!(
                        description.chars().count() <= super::limits::SELECT_OPTION_MAX_CHARS,
                        "String Select option description exceeds {} characters",
                        super::limits::SELECT_OPTION_MAX_CHARS
                    );
                    rendered = rendered.description(description);
                }
                if let Some(emoji) = &option.emoji {
                    if let Some(emoji) = super::value::resolve(emoji, ctx.clone(), data.clone())
                        .await.context("Resolving String Select option emoji")? {
                        rendered = rendered.emoji(serenity::all::ReactionType::try_from(emoji.as_str())?);
                    }
                }
                options.push(rendered);
            }
            (
                menu.id.as_str(),
                match &menu.placeholder {
                    Some(value) => Some(super::value::resolve(value, ctx.clone(), data.clone())
                        .await.context("Resolving String Select placeholder")?),
                    None => None,
                },
                super::value::resolve(&menu.disabled, ctx.clone(), data.clone())
                    .await.context("Resolving String Select disabled state")?,
                menu.min_values,
                menu.max_values,
                CreateSelectMenuKind::String { options: options.into() },
            )
        }
        M::User(menu) => (
            menu.id.as_str(),
            match &menu.placeholder {
                Some(value) => Some(super::value::resolve(value, ctx.clone(), data.clone())
                    .await.context("Resolving User Select placeholder")?),
                None => None,
            },
            super::value::resolve(&menu.disabled, ctx.clone(), data.clone())
                .await.context("Resolving User Select disabled state")?,
            menu.min_values,
            menu.max_values,
            CreateSelectMenuKind::User { default_users: None },
        ),
        M::Role(menu) => (
            menu.id.as_str(),
            match &menu.placeholder {
                Some(value) => Some(super::value::resolve(value, ctx.clone(), data.clone())
                    .await.context("Resolving Role Select placeholder")?),
                None => None,
            },
            super::value::resolve(&menu.disabled, ctx.clone(), data.clone())
                .await.context("Resolving Role Select disabled state")?,
            menu.min_values,
            menu.max_values,
            CreateSelectMenuKind::Role { default_roles: None },
        ),
        M::Mentionable(menu) => (
            menu.id.as_str(),
            match &menu.placeholder {
                Some(value) => Some(super::value::resolve(value, ctx.clone(), data.clone())
                    .await.context("Resolving Mentionable Select placeholder")?),
                None => None,
            },
            super::value::resolve(&menu.disabled, ctx.clone(), data.clone())
                .await.context("Resolving Mentionable Select disabled state")?,
            menu.min_values,
            menu.max_values,
            CreateSelectMenuKind::Mentionable {
                default_users: None,
                default_roles: None,
            },
        ),
        M::Channel(menu) => (
            menu.id.as_str(),
            match &menu.placeholder {
                Some(value) => Some(super::value::resolve(value, ctx.clone(), data.clone())
                    .await.context("Resolving Channel Select placeholder")?),
                None => None,
            },
            super::value::resolve(&menu.disabled, ctx.clone(), data.clone())
                .await.context("Resolving Channel Select disabled state")?,
            menu.min_values,
            menu.max_values,
            CreateSelectMenuKind::Channel {
                channel_types: if menu.channel_types.is_empty() {
                    None
                } else {
                    Some(menu.channel_types.clone().into())
                },
                default_channels: None,
            },
        ),
    };

    if let Some(placeholder) = &placeholder {
        anyhow::ensure!(
            placeholder.chars().count() <= super::limits::SELECT_PLACEHOLDER_MAX_CHARS,
            "Selection menu placeholder exceeds {} characters",
            super::limits::SELECT_PLACEHOLDER_MAX_CHARS
        );
    }

    let custom_id = persist_select_execution(
        execution_id, definition_id, serialized_data, routes,
    )?;
    let mut rendered = CreateSelectMenu::new(custom_id, kind).disabled(disabled);
    if let Some(placeholder) = placeholder {
        rendered = rendered.placeholder(placeholder);
    }
    if let Some(min_values) = min_values {
        rendered = rendered.min_values(min_values);
    }
    if let Some(max_values) = max_values {
        rendered = rendered.max_values(max_values);
    }
    Ok(rendered.into_owned())
}

async fn render_media_gallery<T, D>(
    gallery: &super::component::MediaGallery<T, D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
) -> anyhow::Result<CreateMediaGallery<'static>>
where
    T: Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    anyhow::ensure!(
        !gallery.items.is_empty(),
        "Media Gallery must contain at least one item"
    );
    anyhow::ensure!(
        gallery.items.len() <= super::limits::MEDIA_GALLERY_MAX_ITEMS,
        "Media Gallery has {} items; maximum is {}",
        gallery.items.len(),
        super::limits::MEDIA_GALLERY_MAX_ITEMS
    );
    let mut items = Vec::with_capacity(gallery.items.len());
    for item in &gallery.items {
        let url = super::value::resolve(&item.url, ctx.clone(), data.clone())
            .await.context("Resolving Media Gallery URL")?;
        let description = match &item.description {
            Some(value) => {
                let description = super::value::resolve(value, ctx.clone(), data.clone())
                    .await.context("Resolving Media Gallery description")?;
                anyhow::ensure!(
                    description.chars().count() <= 1_024,
                    "Media Gallery item description exceeds 1024 characters"
                );
                Some(description)
            }
            None => None,
        };
        let spoiler = super::value::resolve(&item.spoiler, ctx.clone(), data.clone())
            .await.context("Resolving Media Gallery spoiler state")?;
        let mut rendered = CreateMediaGalleryItem::new(CreateUnfurledMediaItem::new(url));
        if let Some(description) = description {
            rendered = rendered.description(description);
        }
        rendered = rendered.spoiler(spoiler);
        items.push(rendered);
    }
    Ok(CreateMediaGallery::new(items).into_owned())
}

async fn render_container<T, D>(
    container: &super::component::Container<T, D>,
    app: &App<D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    execution_ids: &mut HashSet<String>,
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<(CreateContainer<'static>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    use super::component::{Component as C, ContainerItem};
    let mut children = Vec::new();
    let mut budget = ComponentBudget::default();

    async fn append_component<T, D>(
        component: &C<T, D>,
        children: &mut Vec<CreateContainerComponent<'static>>,
        budget: &mut ComponentBudget,
        app: &App<D>,
        ctx: &RenderCtx<D>,
        data: &Arc<T>,
        serialized_data: &[u8],
        definition_id: &str,
        execution_ids: &mut HashSet<String>,
        routes: &mut Vec<RoutingRecord>,
    ) -> anyhow::Result<()>
    where
        T: DeserializeOwned + Send + Sync + 'static,
        D: Clone + Send + Sync + 'static,
    {
        let (rendered, child_budget) = Box::pin(render_component(
            component, app, ctx, data, serialized_data, definition_id,
            execution_ids, routes,
        )).await?;
        let rendered_count = rendered.len();
        for part in rendered {
            let child = match part.component {
                CreateComponent::ActionRow(row) => CreateContainerComponent::ActionRow(row),
                CreateComponent::Section(section) => CreateContainerComponent::Section(section),
                CreateComponent::TextDisplay(text) => CreateContainerComponent::TextDisplay(text),
                CreateComponent::MediaGallery(gallery) => CreateContainerComponent::MediaGallery(gallery),
                CreateComponent::File(file) => CreateContainerComponent::File(file),
                CreateComponent::Separator(separator) => CreateContainerComponent::Separator(separator),
                CreateComponent::Container(_) => anyhow::bail!(
                    "Discord does not allow a Container inside another Container"
                ),
                CreateComponent::Label(_) => anyhow::bail!(
                    "Labels cannot be rendered inside a message Container"
                ),
            };
            children.push(child);
        }
        budget.total_components = budget.total_components.saturating_add(child_budget.total_components);
        budget.text_display_chars = budget.text_display_chars.saturating_add(child_budget.text_display_chars);
        budget.direct_container_children = budget.direct_container_children.saturating_add(rendered_count);
        Ok(())
    }

    for item in &container.children {
        match item {
            ContainerItem::Component(component) => {
                append_component(
                    component, &mut children, &mut budget, app, ctx, data, serialized_data,
                    definition_id, execution_ids, routes,
                ).await?;
            }
            ContainerItem::Frame(frame) => {
                let elements = (frame.resolver)(ctx.clone(), data.clone()).await
                    .context("Resolving Frame inside Container")?;
                validate_frame_elements(&elements, execution_ids)
                    .context("Validating Container Frame elements")?;
                for element in elements {
                    register_component_executions(&element.component, app, definition_id)?;
                    append_component(
                        &element.component, &mut children, &mut budget, app, ctx, data,
                        serialized_data, definition_id, execution_ids, routes,
                    ).await.context("Rendering an element from a Container Frame")?;
                }
            }
        }
    }

    anyhow::ensure!(
        children.len() <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN,
        "Rendered Container has {} direct children; maximum is {}",
        children.len(), super::limits::CONTAINER_MAX_DIRECT_CHILDREN
    );
    let direct_child_count = children.len();
    let mut rendered = CreateContainer::new(children);
    if let Some(accent) = &container.accent_color {
        let accent = super::value::resolve(accent, ctx.clone(), data.clone())
            .await.context("Resolving Container accent color")?;
        rendered = rendered.accent_color(serenity::model::prelude::Colour::new(accent));
    }
    rendered = rendered.spoiler(container.spoiler);
    anyhow::ensure!(
        direct_child_count <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN,
        "Rendered Container has {direct_child_count} direct children; maximum is {}",
        super::limits::CONTAINER_MAX_DIRECT_CHILDREN
    );
    budget.total_components = budget.total_components.saturating_add(1);
    budget.direct_container_children = direct_child_count;
    Ok((rendered.into_owned(), budget))
}

async fn render_component<T, D>(
    component: &Component<T, D>,
    app: &App<D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    execution_ids: &mut HashSet<String>,
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<(Vec<RenderedComponent>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    use super::component::Component as C;

    match component {
        C::Text(text) => {
            let content = super::value::resolve(&text.content, ctx.clone(), data.clone())
                .await.context("Resolving Text Display content")?;
            let chars = content.chars().count();
            anyhow::ensure!(
                chars <= super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS,
                "Text Display has {chars} characters; maximum is {}",
                super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS
            );
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::TextDisplay(CreateTextDisplay::new(content).into_owned()),
                }],
                ComponentBudget { total_components: 1, text_display_chars: chars, direct_container_children: 0 },
            ))
        }
        C::Button(button) => {
            let rendered = render_button(
                button, ctx, data, definition_id, serialized_data, routes,
            ).await?;
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::ActionRow(CreateActionRow::buttons(vec![rendered]).into()),
                }],
                ComponentBudget { total_components: 2, text_display_chars: 0, direct_container_children: 0 },
            ))
        }
        C::ButtonRow(row) => {
            let mut buttons = Vec::with_capacity(row.populated());
            for slot in [&row.slots.0, &row.slots.1, &row.slots.2, &row.slots.3, &row.slots.4] {
                match slot {
                    super::component::ButtonSlot::Button(button) => {
                        buttons.push(render_button(
                            button, ctx, data, definition_id, serialized_data, routes,
                        ).await?);
                    }
                    super::component::ButtonSlot::LinkButton(button) => {
                        buttons.push(render_link_button(button, ctx, data).await?);
                    }
                    super::component::ButtonSlot::Empty => {}
                }
            }
            anyhow::ensure!(!buttons.is_empty(), "Button Action Row cannot be empty");
            let count = buttons.len();
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::ActionRow(CreateActionRow::buttons(buttons).into()),
                }],
                ComponentBudget { total_components: 1 + count, text_display_chars: 0, direct_container_children: 0 },
            ))
        }
        C::LinkButton(button) => {
            let rendered = render_link_button(button, ctx, data).await?;
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::ActionRow(CreateActionRow::buttons(vec![rendered]).into()),
                }],
                ComponentBudget { total_components: 2, text_display_chars: 0, direct_container_children: 0 },
            ))
        }
        C::Section(section) => {
            use super::value::Slot;

            let mut text_components = Vec::with_capacity(3);
            let mut text_chars = 0usize;

            for slot in [&section.text.0, &section.text.1, &section.text.2] {
                if let Slot::Value(value) = slot {
                    let content = super::value::resolve(value, ctx.clone(), data.clone())
                        .await.context("Resolving Section Text Display content")?;
                    text_chars = text_chars.saturating_add(content.chars().count());
                    text_components.push(CreateSectionComponent::TextDisplay(
                        CreateTextDisplay::new(content).into_owned()
                    ));
                }
            }

            anyhow::ensure!(
                !text_components.is_empty(),
                "Section must contain at least one Text Display"
            );
            anyhow::ensure!(
                text_chars <= super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS,
                "Section Text Displays exceed the message text budget"
            );

            let accessory = match &section.accessory {
                Some(super::component::SectionAccessory::Button(button)) => {
                    let button = render_button(
                        button, ctx, data, definition_id, serialized_data, routes,
                    ).await.with_context(|| "Rendering Section Button accessory")?;
                    CreateSectionAccessory::Button(button)
                }
                Some(super::component::SectionAccessory::Thumbnail(thumbnail)) => {
                    let url = super::value::resolve(
                        &thumbnail.url, ctx.clone(), data.clone()
                    ).await.context("Resolving Section Thumbnail URL")?;
                    let description = match &thumbnail.description {
                        Some(value) => Some(
                            super::value::resolve(value, ctx.clone(), data.clone())
                                .await.context("Resolving Section Thumbnail description")?
                        ),
                        None => None,
                    };
                    let spoiler = super::value::resolve(
                        &thumbnail.spoiler, ctx.clone(), data.clone()
                    ).await.context("Resolving Section Thumbnail spoiler state")?;

                    if let Some(description) = &description {
                        anyhow::ensure!(
                            description.chars().count() <= 1_024,
                            "Thumbnail description exceeds 1024 characters"
                        );
                    }
                    let mut rendered = CreateThumbnail::new(CreateUnfurledMediaItem::new(url));
                    if let Some(description) = description {
                        rendered = rendered.description(description);
                    }
                    if spoiler {
                        rendered = rendered.spoiler(true);
                    }
                    CreateSectionAccessory::Thumbnail(rendered.into_owned())
                }
                None => anyhow::bail!(
                    "Section requires a Button or Thumbnail accessory"
                ),
            };

            let text_component_count = text_components.len();
            let rendered = CreateSection::new(text_components, accessory).into_owned();
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::Section(rendered),
                }],
                ComponentBudget {
                    total_components: 2 + text_component_count,
                    text_display_chars: text_chars,
                    direct_container_children: 0,
                },
            ))
        },
        C::Container(container) => {
            let (rendered, budget) = render_container(
                container, app, ctx, data, serialized_data, definition_id, execution_ids, routes,
            ).await.context("Rendering Container")?;
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::Container(rendered),
                }],
                budget,
            ))
        }
        C::Separator(separator) => {
            let spacing = super::value::resolve(
                &separator.spacing, ctx.clone(), data.clone()
            ).await.context("Resolving Separator spacing")?;
            let divider = super::value::resolve(
                &separator.divider, ctx.clone(), data.clone()
            ).await.context("Resolving Separator divider")?;
            let spacing = match spacing {
                super::component::SeparatorSpacing::Small =>
                    serenity::model::prelude::SeparatorSpacingSize::Small,
                super::component::SeparatorSpacing::Large =>
                    serenity::model::prelude::SeparatorSpacingSize::Large,
            };
            let rendered = CreateSeparator::new().spacing(spacing).divider(divider);
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::Separator(rendered),
                }],
                ComponentBudget {
                    total_components: 1,
                    text_display_chars: 0,
                    direct_container_children: 0,
                },
            ))
        },
        C::MediaGallery(gallery) => {
            let rendered = render_media_gallery(gallery, ctx, data)
                .await.context("Rendering Media Gallery")?;
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::MediaGallery(rendered),
                }],
                ComponentBudget {
                    total_components: 1,
                    text_display_chars: 0,
                    direct_container_children: 0,
                },
            ))
        }
        C::File(file) => {
            let filename = file
                .attachment_uri
                .strip_prefix("attachment://")
                .ok_or_else(|| anyhow::anyhow!(
                    "File component requires a complete attachment:// URI"
                ))?;
            anyhow::ensure!(
                !filename.trim().is_empty()
                    && !filename.contains('/')
                    && !filename.contains('\\'),
                "File component requires an attachment URI with a filename and no path separators"
            );
            let rendered = CreateFile::new(CreateUnfurledMediaItem::new(
                file.attachment_uri.clone()
            )).spoiler(file.spoiler).into_owned();
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::File(rendered),
                }],
                ComponentBudget {
                    total_components: 1,
                    text_display_chars: 0,
                    direct_container_children: 0,
                },
            ))
        }
        C::Select(menu) => {
            let rendered = render_select_menu(
                menu, ctx, data, serialized_data, definition_id, routes,
            ).await.context("Rendering Selection menu")?;
            Ok((
                vec![RenderedComponent {
                    component: CreateComponent::ActionRow(
                        CreateActionRow::select_menu(rendered).into()
                    ),
                }],
                ComponentBudget {
                    total_components: 2,
                    text_display_chars: 0,
                    direct_container_children: 0,
                },
            ))
        },
    }
}

fn validate_frame_elements<T, D>(
    elements: &[super::frame::Element<T, D>],
    execution_ids: &mut HashSet<String>,
) -> anyhow::Result<()> {
    for element in elements {
        validate_component_ids(&element.component, execution_ids)
            .context("Validating interactive execution IDs in Frame")?;
    }
    Ok(())
}

async fn render_frame_elements<T, D>(
    elements: &[super::frame::Element<T, D>],
    app: &App<D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    execution_ids: &mut HashSet<String>,
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<(Vec<RenderedComponent>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    validate_frame_elements(elements, execution_ids)?;
    let mut rendered = Vec::new();
    let mut budget = ComponentBudget::default();
    for element in elements {
        register_component_executions(&element.component, app, definition_id)?;
        let (mut parts, item_budget) = render_component(
            &element.component, app, ctx, data, serialized_data, definition_id,
            execution_ids, routes,
        ).await?;
        rendered.append(&mut parts);
        budget = add_budgets(budget, item_budget);
    }
    Ok((rendered, budget))
}

async fn render_page_items<T, D>(
    items: &[PageItem<T, D>],
    app: &App<D>,
    ctx: &RenderCtx<D>,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    execution_ids: &mut HashSet<String>,
    routes: &mut Vec<RoutingRecord>,
) -> anyhow::Result<(Vec<RenderedComponent>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
    D: Clone + Send + Sync + 'static,
{
    let mut rendered = Vec::new();
    let mut budget = ComponentBudget::default();
    for item in items {
        match item {
            PageItem::Component(component) => {
                let (mut parts, item_budget) = render_component(
                    component, app, ctx, data, serialized_data, definition_id,
                    execution_ids, routes,
                ).await?;
                rendered.append(&mut parts);
                budget = add_budgets(budget, item_budget);
            }
            PageItem::Frame(frame) => {
                let elements = (frame.resolver)(ctx.clone(), data.clone()).await
                    .context("Resolving Frame")?;
                let (mut parts, item_budget) = render_frame_elements(
                    &elements, app, ctx, data, serialized_data, definition_id,
                    execution_ids, routes,
                ).await?;
                rendered.append(&mut parts);
                budget = add_budgets(budget, item_budget);
            }
        }
    }
    Ok((rendered, budget))
}

impl<T, D> RegisteredPage<T, D>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Returns the registered Page ID.
    pub fn id(&self) -> &str { &self.definition.id }

    /// Resolves the Page and its Frames once, validates the completed result,
    /// persists interaction routes, and returns exactly one message.
    pub async fn render(&self, app: &App<D>, data: T) -> anyhow::Result<RenderedMessage>
    where
        D: Clone + Send + Sync + 'static,
    {
        let data = Arc::new(data);
        let serialized_data = serde_json::to_vec(data.as_ref())
            .with_context(|| format!("Serializing data for Page '{}'", self.id()))?;
        let ctx = app.render_context(self.id());

        // Reserve static IDs first. Dynamic Frame output uses the same namespace.
        let mut execution_ids = HashSet::new();
        for item in &self.definition.items {
            if let PageItem::Component(component) = item {
                validate_component_ids(component, &mut execution_ids)?;
            }
        }

        let mut routes = Vec::new();
        let (components, budget) = render_page_items(
            &self.definition.items, app, &ctx, &data, &serialized_data, self.id(),
            &mut execution_ids, &mut routes,
        ).await.with_context(|| format!(
            "Expanding components and Frames for Page '{}'", self.id()
        ))?;

        validate_message_budget(budget)
            .with_context(|| format!("Validating Page '{}'", self.id()))?;

        // A failed property, Frame resolver, or limit check leaves no route records.
        for record in routes {
            let execution_id = record.execution_id.clone();
            app.store().insert(record).await.with_context(|| format!(
                "Persisting execution '{}' for Page '{}'", execution_id, self.id()
            ))?;
        }

        Ok(RenderedMessage { components, attachments: Vec::new() })
    }
}
