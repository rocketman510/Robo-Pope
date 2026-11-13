//! Page<T> builder and immutable RegisteredPage<T> typestate.
use std::sync::Arc;
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
use crate::app::store::{derive_custom_id, DefinitionKind, StoreRecord};
use super::component::*;
use super::frame::Frame;
use super::frame::pagination::PaginationRequest;
use super::render::{RenderedComponent, RenderedMessage, RenderedPage};
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

fn register_component_executions<T>(
    component: &Component<T>,
    app: &App,
    definition_id: &str,
    frame_identity: Option<&str>,
) -> anyhow::Result<()>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    use super::component::{ContainerItem, SectionAccessory};
    match component {
        Component::Button(button) => button.execution.register(app, definition_id, frame_identity)?,
        Component::ButtonRow(row) => {
            for button in [
                row.slots.0.as_ref(), row.slots.1.as_ref(), row.slots.2.as_ref(),
                row.slots.3.as_ref(), row.slots.4.as_ref(),
            ].into_iter().flatten() {
                button.execution.register(app, definition_id, frame_identity)?;
            }
        }
        Component::Section(section) => {
            if let Some(SectionAccessory::Button(button)) = &section.accessory {
                button.execution.register(app, definition_id, frame_identity)?;
            }
        }
        Component::Select(menu) => match menu {
            SelectMenu::String(menu) => menu.execution.register(app, definition_id, frame_identity)?,
            SelectMenu::User(menu) => menu.execution.register(app, definition_id, frame_identity)?,
            SelectMenu::Role(menu) => menu.execution.register(app, definition_id, frame_identity)?,
            SelectMenu::Mentionable(menu) => menu.execution.register(app, definition_id, frame_identity)?,
            SelectMenu::Channel(menu) => menu.execution.register(app, definition_id, frame_identity)?,
        },
        Component::Container(container) => {
            for child in &container.children {
                if let ContainerItem::Component(component) = child {
                    register_component_executions(component, app, definition_id, frame_identity)?;
                }
            }
        }
        Component::Text(_) | Component::Separator(_) | Component::MediaGallery(_)
        | Component::File(_) | Component::LinkButton(_) => {}
    }
    Ok(())
}

fn contains_interactive_component<T>(component: &Component<T>) -> bool {
    use super::component::{ContainerItem, SectionAccessory};
    match component {
        Component::Button(_) | Component::ButtonRow(_) | Component::Select(_) => true,
        Component::Section(section) => matches!(&section.accessory, Some(SectionAccessory::Button(_))),
        Component::Container(container) => container.children.iter().any(|child| match child {
            ContainerItem::Component(component) => contains_interactive_component(component),
            ContainerItem::Frame(_) => false,
        }),
        Component::Text(_) | Component::Separator(_) | Component::MediaGallery(_)
        | Component::File(_) | Component::LinkButton(_) => false,
    }
}

fn append_frame_identity(parent: Option<&str>, frame_id: &str, key: &str) -> String {
    match parent {
        Some(parent) => format!("{parent}:{frame_id}:{key}"),
        None => format!("{frame_id}:{key}"),
    }
}

fn pagination_identity(frame_id: &str, page_index: usize) -> String {
    if page_index == 0 { frame_id.to_owned() } else { format!("{frame_id}:page:{page_index}") }
}

fn pagination_execution_id(frame_id: &str, direction: &str) -> String {
    format!("__frame:{frame_id}:{direction}")
}

fn collect_component_frames<T>(
    component: &Component<T>,
    page_paginate: bool,
    output: &mut Vec<String>,
) {
    use super::component::ContainerItem;
    if let Component::Container(container) = component {
        for child in &container.children {
            match child {
                ContainerItem::Component(component) => collect_component_frames(component, page_paginate, output),
                ContainerItem::Frame(frame) if frame.paginate || page_paginate => output.push(frame.id.clone()),
                ContainerItem::Frame(_) => {}
            }
        }
    }
}

fn collect_paginated_frames_in_component<T>(
    component: &Component<T>,
    page_paginate: bool,
    output: &mut Vec<(usize, String)>,
) {
    use super::component::ContainerItem;
    if let Component::Container(container) = component {
        for child in &container.children {
            match child {
                ContainerItem::Frame(frame) if frame.paginate || page_paginate => {
                    output.push((usize::MAX, frame.id.clone()));
                }
                ContainerItem::Component(component) => {
                    collect_paginated_frames_in_component(component, page_paginate, output)
                }
                ContainerItem::Frame(_) => {}
            }
        }
    }
}

fn collect_paginated_frames<T>(definition: &PageDefinition<T>) -> Vec<String> {
    use super::component::ContainerItem;
    let mut output = Vec::new();
    for item in &definition.items {
        match item {
            PageItem::Frame(frame) if frame.paginate || definition.paginate => output.push(frame.id.clone()),
            PageItem::Component(component) => collect_component_frames(component, definition.paginate, &mut output),
            PageItem::Frame(_) => {}
        }
        if let PageItem::Component(Component::Container(container)) = item {
            for child in &container.children {
                if let ContainerItem::Frame(frame) = child {
                    if frame.paginate || definition.paginate { output.push(frame.id.clone()); }
                }
            }
        }
    }
    output.sort();
    output.dedup();
    output
}

fn register_pagination_callbacks<T>(
    app: &App,
    definition: Arc<PageDefinition<T>>,
    frame_ids: &[String],
) -> anyhow::Result<()>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    for frame_id in frame_ids {
        for direction in ["previous", "next"] {
            let execution_id = pagination_execution_id(frame_id, direction);
            let callback_definition = definition.clone();
            let callback_frame_id = frame_id.clone();
            let callback_direction = direction.to_owned();
            let execution = super::execution::Execution::with_input(
                execution_id,
                move |ctx, data: T, request: PaginationRequest| {
                    let definition = callback_definition.clone();
                    let frame_id = callback_frame_id.clone();
                    let direction = callback_direction.clone();
                    async move {
                        anyhow::ensure!(
                            request.frame_id == frame_id,
                            "Pagination request refers to the wrong Frame"
                        );
                        anyhow::ensure!(
                            request.source_page_index < request.page_count
                                && request.target_page_index < request.page_count,
                            "Pagination request is outside the Frame's page range"
                        );
                        anyhow::ensure!(
                            (direction == "next" && request.target_page_index == request.source_page_index + 1)
                                || (direction == "previous" && request.target_page_index + 1 == request.source_page_index),
                            "Pagination request direction does not match its target page"
                        );

                        let mut next_state = request.expected_state.clone();
                        next_state["page_index"] = serde_json::json!(request.target_page_index);
                        let updated = ctx.app.store()
                            .compare_and_set_pagination_state(
                                &request.custom_id,
                                &request.expected_state,
                                next_state,
                            )
                            .await?;
                        anyhow::ensure!(updated, "This pagination control is stale or has already been used");

                        // Invalidate controls belonging to every other page. The
                        // clicked record remains as the FIRST-observable state record;
                        // the target page's controls are recreated during rendering.
                        for page_index in 0..request.page_count {
                            let identity = pagination_identity(&frame_id, page_index);
                            for name in ["previous", "next"] {
                                let execution_id = pagination_execution_id(&frame_id, name);
                                let candidate = derive_custom_id(
                                    &definition.id,
                                    &execution_id,
                                    &serde_json::to_vec(&data)?,
                                    Some(&identity),
                                )?;
                                if candidate != request.custom_id {
                                    ctx.app.store().remove(&candidate).await?;
                                }
                            }
                        }

                        let page = RegisteredPage { definition };
                        let rendered = page.render_pages(&ctx.app, data).await?;
                        let message = rendered.pages.into_iter()
                            .nth(request.target_page_index)
                            .ok_or_else(|| anyhow::anyhow!("Rendered pagination page does not exist"))?;
                        ctx.edit_message(message).await
                    }
                },
            );
            execution.register(app, &definition.id, Some(frame_id))?;
        }
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
        app.reserve_definition_id(&self.id)?;
        for item in &self.items {
            if let PageItem::Component(component) = item {
                if let Err(error) = register_component_executions(component, app, &self.id, None) {
                    app.executions.remove_definition(&self.id);
                    app.release_definition_id(&self.id);
                    return Err(error);
                }
            }
        }
        let definition = Arc::new(PageDefinition {
            id: self.id,
            items: self.items,
            paginate: self.paginate,
        });
        let frame_ids = collect_paginated_frames(&definition);
        if let Err(error) = register_pagination_callbacks(app, definition.clone(), &frame_ids) {
            app.executions.remove_definition(&definition.id);
            app.release_definition_id(&definition.id);
            return Err(error);
        }
        if let Err(error) = app.pages.insert(definition.clone()) {
            app.executions.remove_definition(&definition.id);
            app.release_definition_id(&definition.id);
            return Err(error);
        }
        Ok(RegisteredPage { definition })
    }
}


use super::frame::pagination::ComponentBudget;

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

async fn persist_button_execution<T>(
    app: &App,
    button: &Button<T>,
    definition_id: &str,
    serialized_data: &[u8],
    frame_identity: Option<&str>,
) -> anyhow::Result<String> {
    let custom_id = derive_custom_id(
        definition_id,
        &button.execution.id,
        serialized_data,
        frame_identity,
    )?;
    app.store().insert(StoreRecord {
        custom_id: custom_id.0.clone(),
        definition_kind: DefinitionKind::Page,
        definition_id: definition_id.to_owned(),
        execution_id: button.execution.id.clone(),
        serialized_data: serialized_data.to_vec(),
        frame_identity: frame_identity.map(str::to_owned),
        pagination_state: None,
        schema_version: 1,
    }).await.with_context(|| format!(
        "Persisting execution '{}' for Page '{}'",
        button.execution.id, definition_id
    ))?;
    Ok(custom_id.0)
}

async fn persist_select_execution(
    app: &App,
    execution_id: &str,
    definition_id: &str,
    serialized_data: &[u8],
    frame_identity: Option<&str>,
) -> anyhow::Result<String> {
    let custom_id = derive_custom_id(
        definition_id, execution_id, serialized_data, frame_identity,
    )?;
    app.store().insert(StoreRecord {
        custom_id: custom_id.0.clone(),
        definition_kind: DefinitionKind::Page,
        definition_id: definition_id.to_owned(),
        execution_id: execution_id.to_owned(),
        serialized_data: serialized_data.to_vec(),
        frame_identity: frame_identity.map(str::to_owned),
        pagination_state: None,
        schema_version: 1,
    }).await.with_context(|| format!(
        "Persisting selection execution '{execution_id}' for Page '{definition_id}'"
    ))?;
    Ok(custom_id.0)
}

async fn render_button<T>(
    app: &App,
    button: &Button<T>,
    ctx: &RenderCtx,
    data: &Arc<T>,
    definition_id: &str,
    serialized_data: &[u8],
    frame_identity: Option<&str>,
) -> anyhow::Result<CreateButton<'static>>
where
    T: Send + Sync + 'static,
{
    let label = super::value::resolve(&button.label, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving label for Button '{}'", button.id))?;
    let style = super::value::resolve(&button.style, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving style for Button '{}'", button.id))?;
    let disabled = super::value::resolve(&button.disabled, ctx.clone(), data.clone())
        .await.with_context(|| format!("Resolving disabled state for Button '{}'", button.id))?;
    let custom_id = persist_button_execution(
        app, button, definition_id, serialized_data, frame_identity,
    ).await?;

    let serenity_style = match style {
        super::component::ButtonStyle::Primary => SerenityButtonStyle::Primary,
        super::component::ButtonStyle::Secondary => SerenityButtonStyle::Secondary,
        super::component::ButtonStyle::Success => SerenityButtonStyle::Success,
        super::component::ButtonStyle::Danger => SerenityButtonStyle::Danger,
    };

    Ok(CreateButton::new(custom_id)
        .label(label)
        .style(serenity_style)
        .disabled(disabled)
        .into_owned())
}

async fn render_select_menu<T>(
    menu: &SelectMenu<T>,
    app: &App,
    ctx: &RenderCtx,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    frame_identity: Option<&str>,
) -> anyhow::Result<CreateSelectMenu<'static>>
where
    T: Send + Sync + 'static,
{
    use super::select::SelectMenu as M;

    let (execution_id, placeholder, disabled, min_values, max_values, kind) = match menu {
        M::String(menu) => {
            let mut options = Vec::with_capacity(menu.options.len());
            for option in &menu.options {
                let label = super::value::resolve(&option.label, ctx.clone(), data.clone())
                    .await.context("Resolving String Select option label")?;
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
                    rendered = rendered.description(description);
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

    let custom_id = persist_select_execution(
        app, execution_id, definition_id, serialized_data, frame_identity,
    ).await?;
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

async fn render_media_gallery<T>(
    gallery: &super::component::MediaGallery<T>,
    ctx: &RenderCtx,
    data: &Arc<T>,
) -> anyhow::Result<CreateMediaGallery<'static>>
where
    T: Send + Sync + 'static,
{
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
            Some(value) => Some(super::value::resolve(value, ctx.clone(), data.clone())
                .await.context("Resolving Media Gallery description")?),
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

async fn render_container<T>(
    container: &super::component::Container<T>,
    app: &App,
    ctx: &RenderCtx,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    frame_identity: Option<&str>,
) -> anyhow::Result<(CreateContainer<'static>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    use super::component::{Component as C, ContainerItem};

    let mut children = Vec::new();
    let mut budget = ComponentBudget::default();

    async fn append_component<T>(
        component: &C<T>,
        key: Option<&str>,
        children: &mut Vec<CreateContainerComponent<'static>>,
        budget: &mut ComponentBudget,
        app: &App,
        ctx: &RenderCtx,
        data: &Arc<T>,
        serialized_data: &[u8],
        definition_id: &str,
    ) -> anyhow::Result<()>
    where
        T: DeserializeOwned + Send + Sync + 'static,
    {
        let (rendered, child_budget) = Box::pin(render_component(
            component, app, ctx, data, serialized_data, definition_id, key,
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
                    component, frame_identity, &mut children, &mut budget,
                    app, ctx, data, serialized_data, definition_id,
                ).await?;
            }
            ContainerItem::Frame(frame) => {
                let elements = (frame.resolver)(ctx.clone(), data.clone()).await
                    .with_context(|| format!("Resolving Frame '{}' inside Container", frame.id))?;
                validate_frame_elements(&elements, &frame.id)?;
                for element in elements {
                    let key = element.key.as_deref()
                        .map(|key| append_frame_identity(frame_identity, &frame.id, key));
                    let needs_key = contains_interactive_component(&element.component);
                    anyhow::ensure!(
                        !needs_key || key.is_some(),
                        "Interactive element in Container Frame '{}' requires a stable developer key",
                        frame.id
                    );
                    register_component_executions(&element.component, app, definition_id, key.as_deref())?;
                    append_component(
                        &element.component, key.as_deref(), &mut children, &mut budget,
                        app, ctx, data, serialized_data, definition_id,
                    ).await.with_context(|| format!(
                        "Rendering element in Container Frame '{}'", frame.id
                    ))?;
                }
            }
        }
    }

    anyhow::ensure!(
        children.len() <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN,
        "Rendered Container has {} direct children; maximum is {}",
        children.len(),
        super::limits::CONTAINER_MAX_DIRECT_CHILDREN
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

async fn render_component<T>(
    component: &Component<T>,
    app: &App,
    ctx: &RenderCtx,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    frame_identity: Option<&str>,
) -> anyhow::Result<(Vec<RenderedComponent>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
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
                app, button, ctx, data, definition_id, serialized_data, frame_identity,
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
            for button in [
                row.slots.0.as_ref(),
                row.slots.1.as_ref(),
                row.slots.2.as_ref(),
                row.slots.3.as_ref(),
                row.slots.4.as_ref(),
            ].into_iter().flatten() {
                buttons.push(render_button(
                    app, button, ctx, data, definition_id, serialized_data, frame_identity,
                ).await?);
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
            let label = super::value::resolve(&button.label, ctx.clone(), data.clone()).await
                .context("Resolving Link Button label")?;
            let url = super::value::resolve(&button.url, ctx.clone(), data.clone()).await
                .context("Resolving Link Button URL")?;
            let disabled = super::value::resolve(&button.disabled, ctx.clone(), data.clone()).await
                .context("Resolving Link Button disabled state")?;
            let rendered = CreateButton::new_link(url).label(label).disabled(disabled).into_owned();
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
                        app, button, ctx, data, definition_id, serialized_data, frame_identity,
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
                    total_components: 1 + text_component_count,
                    text_display_chars: text_chars,
                    direct_container_children: 0,
                },
            ))
        },
        C::Container(container) => {
            let (rendered, budget) = render_container(
                container, app, ctx, data, serialized_data, definition_id, frame_identity,
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
            anyhow::ensure!(
                !file.filename.trim().is_empty() && !file.filename.contains('/') && !file.filename.contains('\\'),
                "File component requires a non-empty filename without path separators"
            );
            let rendered = CreateFile::new(CreateUnfurledMediaItem::new(
                format!("attachment://{}", file.filename)
            )).into_owned();
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
                menu, app, ctx, data, serialized_data, definition_id, frame_identity,
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

async fn render_component_list<T>(
    components: &[&Component<T>],
    app: &App,
    ctx: &RenderCtx,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    frame_identity: Option<&str>,
) -> anyhow::Result<(Vec<RenderedComponent>, ComponentBudget)>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    let mut rendered = Vec::new();
    let mut budget = ComponentBudget::default();
    for component in components {
        let (mut parts, component_budget) = render_component(
            component, app, ctx, data, serialized_data, definition_id, frame_identity,
        ).await?;
        rendered.append(&mut parts);
        budget = add_budgets(budget, component_budget);
    }
    Ok((rendered, budget))
}

fn validate_frame_elements<T>(
    elements: &[super::frame::Element<T>],
    frame_id: &str,
) -> anyhow::Result<()> {
    let mut element_keys = HashSet::new();
    let mut nested_frame_ids = HashSet::new();
    for element in elements {
        if let Some(key) = element.key.as_deref() {
            anyhow::ensure!(!key.trim().is_empty(),
                "Frame '{frame_id}' contains an empty element key");
            anyhow::ensure!(element_keys.insert(key.to_owned()),
                "Frame '{frame_id}' contains duplicate element key '{key}'");
        }
        let mut execution_ids = HashSet::new();
        validate_component_ids(&element.component, &mut execution_ids, &mut nested_frame_ids)?;
        let interactive = contains_interactive_component(&element.component);
        anyhow::ensure!(
            !interactive || element.key.as_ref().is_some_and(|key| !key.trim().is_empty()),
            "Interactive Frame element in Frame '{frame_id}' requires a non-empty stable developer key"
        );
    }
    Ok(())
}

struct RenderedFrameElement {
    components: Vec<RenderedComponent>,
    budget: ComponentBudget,
}

async fn render_frame_elements<T>(
    elements: &[super::frame::Element<T>],
    app: &App,
    ctx: &RenderCtx,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    frame_id: &str,
) -> anyhow::Result<Vec<RenderedFrameElement>>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    validate_frame_elements(elements, frame_id)?;
    let mut rendered = Vec::with_capacity(elements.len());
    for element in elements {
        let frame_identity = element.key.as_deref()
            .map(|key| append_frame_identity(None, frame_id, key));
        let has_execution = contains_interactive_component(&element.component);
        anyhow::ensure!(
            !has_execution || frame_identity.is_some(),
            "Interactive Frame element in Frame '{frame_id}' requires a stable developer key"
        );
        let frame_identity = frame_identity.as_deref();
        register_component_executions(&element.component, app, definition_id, frame_identity)?;
        let (components, budget) = render_component(
            &element.component, app, ctx, data, serialized_data, definition_id, frame_identity,
        ).await?;
        rendered.push(RenderedFrameElement { components, budget });
    }
    Ok(rendered)
}

fn message_budget_fits(budget: ComponentBudget) -> bool {
    budget.total_components <= super::limits::MESSAGE_MAX_COMPONENTS
        && budget.text_display_chars <= super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS
        && budget.direct_container_children <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN
}

fn plan_rendered_frame(
    elements: Vec<RenderedFrameElement>,
    reserved: ComponentBudget,
) -> anyhow::Result<(Vec<(Vec<RenderedComponent>, ComponentBudget)>, usize)> {
    anyhow::ensure!(
        message_budget_fits(reserved),
        "Static content exceeds the Discord message budget"
    );
    let mut pages = Vec::new();
    let mut current_components = Vec::new();
    let mut current_budget = reserved;
    let mut current_count = 0usize;

    for element in elements {
        anyhow::ensure!(
            message_budget_fits(add_budgets(reserved, element.budget)),
            "A single resolved Frame element cannot fit within the remaining Discord message budget"
        );
        let combined = add_budgets(current_budget, element.budget);
        if !message_budget_fits(combined) && current_count > 0 {
            pages.push((std::mem::take(&mut current_components), current_budget));
            current_budget = reserved;
            current_count = 0;
        }
        current_budget = add_budgets(current_budget, element.budget);
        current_components.extend(element.components);
        current_count += 1;
    }

    if current_count > 0 || pages.is_empty() {
        pages.push((current_components, current_budget));
    }
    let page_count = pages.len();
    Ok((pages, page_count))
}

async fn create_pagination_navigation(
    app: &App,
    definition_id: &str,
    frame_id: &str,
    serialized_data: &[u8],
    state: &super::frame::pagination::PaginationState,
) -> anyhow::Result<RenderedComponent> {
    let identity = pagination_identity(frame_id, state.page_index);
    let mut buttons = Vec::with_capacity(2);
    for (direction, label, disabled) in [
        ("previous", "Previous", state.page_index == 0),
        ("next", "Next", state.page_index + 1 == state.page_count),
    ] {
        let execution_id = pagination_execution_id(frame_id, direction);
        let custom_id = derive_custom_id(
            definition_id,
            &execution_id,
            serialized_data,
            Some(&identity),
        )?;
        if app.store().resolve(&custom_id).await?.is_none() {
            app.store().insert(StoreRecord {
                custom_id: custom_id.0.clone(),
                definition_kind: DefinitionKind::Page,
                definition_id: definition_id.to_owned(),
                execution_id,
                serialized_data: serialized_data.to_vec(),
                frame_identity: Some(identity.clone()),
                pagination_state: Some(serde_json::to_value(state)?),
                schema_version: 1,
            }).await.with_context(|| format!(
                "Persisting {label} navigation for Frame '{frame_id}'"
            ))?;
        }
        buttons.push(
            CreateButton::new(custom_id.0)
                .label(label)
                .style(SerenityButtonStyle::Primary)
                .disabled(disabled)
                .into_owned()
        );
    }
    Ok(RenderedComponent {
        component: CreateComponent::ActionRow(CreateActionRow::buttons(buttons).into()),
    })
}

fn as_container_child(part: RenderedComponent) -> anyhow::Result<CreateContainerComponent<'static>> {
    match part.component {
        CreateComponent::ActionRow(row) => Ok(CreateContainerComponent::ActionRow(row)),
        CreateComponent::Section(section) => Ok(CreateContainerComponent::Section(section)),
        CreateComponent::TextDisplay(text) => Ok(CreateContainerComponent::TextDisplay(text)),
        CreateComponent::MediaGallery(gallery) => Ok(CreateContainerComponent::MediaGallery(gallery)),
        CreateComponent::File(file) => Ok(CreateContainerComponent::File(file)),
        CreateComponent::Separator(separator) => Ok(CreateContainerComponent::Separator(separator)),
        CreateComponent::Container(_) => anyhow::bail!("Discord does not allow a Container inside another Container"),
        CreateComponent::Label(_) => anyhow::bail!("Labels cannot be rendered inside a message Container"),
    }
}

async fn render_paginated_container<T>(
    container: &super::component::Container<T>,
    app: &App,
    ctx: &RenderCtx,
    data: &Arc<T>,
    serialized_data: &[u8],
    definition_id: &str,
    frame_id: &str,
    page_prefix: Vec<RenderedComponent>,
    page_prefix_budget: ComponentBudget,
    page_suffix: Vec<RenderedComponent>,
    page_suffix_budget: ComponentBudget,
) -> anyhow::Result<RenderedPage>
where
    T: DeserializeOwned + Send + Sync + 'static,
{
    use super::component::ContainerItem;

    let frame_positions: Vec<usize> = container.children.iter().enumerate()
        .filter_map(|(i, child)| match child {
            ContainerItem::Frame(frame) if frame.id == frame_id => Some(i),
            _ => None,
        }).collect();
    anyhow::ensure!(frame_positions.len() == 1, "Paginated Frame '{frame_id}' is not a direct Container child");
    let frame_position = frame_positions[0];
    let frame = match &container.children[frame_position] {
        ContainerItem::Frame(frame) => frame,
        _ => unreachable!(),
    };

    let prefix_refs: Vec<&Component<T>> = container.children[..frame_position].iter()
        .filter_map(|item| match item { ContainerItem::Component(c) => Some(c), ContainerItem::Frame(_) => None })
        .collect();
    let suffix_refs: Vec<&Component<T>> = container.children[frame_position + 1..].iter()
        .filter_map(|item| match item { ContainerItem::Component(c) => Some(c), ContainerItem::Frame(_) => None })
        .collect();

    let (prefix, prefix_budget) = render_component_list(
        &prefix_refs, app, ctx, data, serialized_data, definition_id, None,
    ).await.context("Rendering Container children before nested Frame")?;
    let (suffix, suffix_budget) = render_component_list(
        &suffix_refs, app, ctx, data, serialized_data, definition_id, None,
    ).await.context("Rendering Container children after nested Frame")?;

    let elements = (frame.resolver)(ctx.clone(), data.clone()).await
        .with_context(|| format!("Resolving nested Frame '{frame_id}'"))?;
    let rendered_elements = render_frame_elements(
        &elements, app, ctx, data, serialized_data, definition_id, frame_id,
    ).await?;

    let static_container_children = prefix.len() + suffix.len();
    let mut accent = None;
    if let Some(value) = &container.accent_color {
        accent = Some(super::value::resolve(value, ctx.clone(), data.clone()).await
            .context("Resolving Container accent color")?);
    }

    let mut all_pages: Vec<Vec<RenderedFrameElement>> = Vec::new();
    let mut current: Vec<RenderedFrameElement> = Vec::new();
    let mut current_budget = add_budgets(
        page_prefix_budget,
        add_budgets(page_suffix_budget, add_budgets(
            ComponentBudget { total_components: 1, text_display_chars: 0, direct_container_children: static_container_children + 1 },
            add_budgets(prefix_budget, suffix_budget),
        )),
    );
    let mut current_direct_children = static_container_children + 1; // reserve the navigation Action Row
    let mut current_count = 0usize;

    for element in rendered_elements {
        let direct = element.components.len();
        anyhow::ensure!(
            static_container_children + 1 + direct <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN,
            "A single nested Frame element cannot fit in Container '{frame_id}'"
        );
        let next_budget = add_budgets(current_budget, element.budget);
        let next_direct = current_direct_children + direct;
        let fits = message_budget_fits(next_budget)
            && next_direct <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN;
        if !fits && current_count > 0 {
            all_pages.push(std::mem::take(&mut current));
            current_budget = add_budgets(
                page_prefix_budget,
                add_budgets(page_suffix_budget, add_budgets(
                    ComponentBudget { total_components: 1, text_display_chars: 0, direct_container_children: static_container_children + 1 },
                    add_budgets(prefix_budget, suffix_budget),
                )),
            );
            current_direct_children = static_container_children + 1;
            current_count = 0;
        }
        current_budget = add_budgets(current_budget, element.budget);
        current_direct_children += direct;
        current.push(element);
        current_count += 1;
    }
    if current_count > 0 || all_pages.is_empty() {
        all_pages.push(current);
    }

    // With one page there is no navigation control, so recompute the message
    // validation without its reserved Action Row.
    let page_count = all_pages.len();
    let mut pages = Vec::with_capacity(page_count);
    for (page_index, frame_elements) in all_pages.into_iter().enumerate() {
        let mut children = Vec::new();
        for part in prefix.iter().cloned() { children.push(as_container_child(part)?); }
        for element in frame_elements {
            for part in element.components { children.push(as_container_child(part)?); }
        }
        for part in suffix.iter().cloned() { children.push(as_container_child(part)?); }

        let mut budget = add_budgets(
            page_prefix_budget,
            add_budgets(page_suffix_budget, add_budgets(
                ComponentBudget { total_components: 1, text_display_chars: 0, direct_container_children: children.len() },
                add_budgets(prefix_budget, suffix_budget),
            )),
        );
        if page_count > 1 {
            let state = super::frame::pagination::PaginationState {
                frame_id: frame_id.to_owned(), page_index, page_count,
            };
            let nav = create_pagination_navigation(app, definition_id, frame_id, serialized_data, &state).await?;
            children.push(as_container_child(nav)?);
            budget.total_components = budget.total_components.saturating_add(3);
            budget.direct_container_children = children.len();
        }
        anyhow::ensure!(
            children.len() <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN,
            "Rendered Container has {} direct children; maximum is {}",
            children.len(), super::limits::CONTAINER_MAX_DIRECT_CHILDREN
        );
        validate_message_budget(budget).context("Validating paginated Container message")?;

        let mut builder = CreateContainer::new(children);
        if let Some(accent) = accent {
            builder = builder.accent_color(serenity::model::prelude::Colour::new(accent));
        }
        builder = builder.spoiler(container.spoiler);
        let mut components = page_prefix.clone();
        components.push(RenderedComponent { component: CreateComponent::Container(builder.into_owned()) });
        components.extend(page_suffix.clone());
        pages.push(RenderedMessage {
            components,
            attachments: Vec::new(),
            pagination: (page_count > 1).then_some(super::frame::pagination::PaginationState {
                frame_id: frame_id.to_owned(), page_index, page_count,
            }),
        });
    }
    Ok(RenderedPage { pages, current_index: 0 })
}

impl<T> RegisteredPage<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static
{
    /// Returns the registered Page ID.
    pub fn id(&self) -> &str { &self.definition.id }

    /// Resolves dynamic properties and Frames for one Page instance, persists
    /// execution references, checks Discord limits, and returns the first message.
    pub async fn render(&self, app: &App, data: T)
        -> anyhow::Result<RenderedMessage>
    {
        let mut rendered = self.render_pages(app, data).await
            .with_context(|| format!("Rendering Page '{}'", self.id()))?;
        anyhow::ensure!(!rendered.pages.is_empty(), "Page '{}' rendered no message pages", self.id());
        Ok(rendered.pages.remove(0))
    }

    /// Renders the Page's complete Frame pagination and reports its current position.
    pub async fn render_pages(&self, app: &App, data: T)
        -> anyhow::Result<RenderedPage>
    {
        let data = Arc::new(data);
        let serialized_data = serde_json::to_vec(data.as_ref())
            .with_context(|| format!("Serializing data for Page '{}'", self.id()))?;
        let ctx = app.render_context(self.id());

        let frame_positions: Vec<usize> = self.definition.items.iter().enumerate()
            .filter_map(|(index, item)| matches!(item, PageItem::Frame(_)).then_some(index))
            .collect();
        for (index, item) in self.definition.items.iter().enumerate() {
            if let PageItem::Component(Component::Container(container)) = item {
                let direct_frame_count = container.children.iter().filter(|child|
                    matches!(child, super::component::ContainerItem::Frame(_))
                ).count();
                if direct_frame_count > 1 {
                    anyhow::bail!("Container at Page item {index} has more than one Frame");
                }
            }
        }

        anyhow::ensure!(
            frame_positions.len() <= 1,
            "Page '{}' currently supports one top-level Frame per render",
            self.id()
        );

        let mut nested_frames: Vec<(usize, String)> = Vec::new();
        for (item_index, item) in self.definition.items.iter().enumerate() {
            if let PageItem::Component(component) = item {
                let mut found = Vec::new();
                collect_paginated_frames_in_component(component, self.definition.paginate, &mut found);
                for (_, frame_id) in found { nested_frames.push((item_index, frame_id)); }
            }
        }
        anyhow::ensure!(
            frame_positions.len() + nested_frames.len() <= 1,
            "Page '{}' cannot contain more than one paginated Frame across its component tree",
            self.id()
        );

        if frame_positions.is_empty() && !nested_frames.is_empty() {
            let (container_item_index, frame_id) = nested_frames[0].clone();
            let container = match &self.definition.items[container_item_index] {
                PageItem::Component(Component::Container(container)) => container,
                _ => anyhow::bail!("Paginated nested Frame must be inside a top-level Container"),
            };
            let direct_matches = container.children.iter().filter(|child| {
                matches!(child, super::component::ContainerItem::Frame(frame) if frame.id == frame_id)
            }).count();
            anyhow::ensure!(
                direct_matches == 1,
                "Paginated Frame '{}' must be a direct child of its Container",
                frame_id
            );
            let prefix_refs: Vec<&Component<T>> = self.definition.items[..container_item_index].iter()
                .filter_map(|item| match item { PageItem::Component(component) => Some(component), _ => None })
                .collect();
            let suffix_refs: Vec<&Component<T>> = self.definition.items[container_item_index + 1..].iter()
                .filter_map(|item| match item { PageItem::Component(component) => Some(component), _ => None })
                .collect();
            let (page_prefix, page_prefix_budget) = render_component_list(
                &prefix_refs, app, &ctx, &data, &serialized_data, self.id(), None,
            ).await?;
            let (page_suffix, page_suffix_budget) = render_component_list(
                &suffix_refs, app, &ctx, &data, &serialized_data, self.id(), None,
            ).await?;
            return render_paginated_container(
                container, app, &ctx, &data, &serialized_data, self.id(), &frame_id,
                page_prefix, page_prefix_budget, page_suffix, page_suffix_budget,
            ).await;
        }

        let Some(&frame_position) = frame_positions.first() else {
            let components: Vec<&Component<T>> = self.definition.items.iter()
                .filter_map(|item| match item {
                    PageItem::Component(component) => Some(component),
                    PageItem::Frame(_) => None,
                })
                .collect();
            let (components, budget) = render_component_list(
                &components, app, &ctx, &data, &serialized_data, self.id(), None,
            ).await?;
            validate_message_budget(budget)
                .with_context(|| format!("Validating Page '{}'", self.id()))?;
            return Ok(RenderedPage {
                pages: vec![RenderedMessage {
                    components,
                    attachments: Vec::new(),
                    pagination: None,
                }],
                current_index: 0,
            });
        };

        let frame = match &self.definition.items[frame_position] {
            PageItem::Frame(frame) => frame,
            PageItem::Component(_) => unreachable!("frame position came from a Frame item"),
        };

        let prefix: Vec<&Component<T>> = self.definition.items[..frame_position].iter()
            .filter_map(|item| match item {
                PageItem::Component(component) => Some(component),
                PageItem::Frame(_) => None,
            })
            .collect();
        let suffix: Vec<&Component<T>> = self.definition.items[frame_position + 1..].iter()
            .filter_map(|item| match item {
                PageItem::Component(component) => Some(component),
                PageItem::Frame(_) => None,
            })
            .collect();

        let (prefix_components, prefix_budget) = render_component_list(
            &prefix, app, &ctx, &data, &serialized_data, self.id(), None,
        ).await.with_context(|| format!("Rendering content before Frame '{}'", frame.id))?;
        let (suffix_components, suffix_budget) = render_component_list(
            &suffix, app, &ctx, &data, &serialized_data, self.id(), None,
        ).await.with_context(|| format!("Rendering content after Frame '{}'", frame.id))?;

        let elements = (frame.resolver)(ctx.clone(), data.clone()).await
            .with_context(|| format!("Resolving Frame '{}' in Page '{}'", frame.id, self.id()))?;

        let reserved = add_budgets(prefix_budget, suffix_budget);
        let should_paginate = self.definition.paginate || frame.paginate;

        let rendered_elements = render_frame_elements(
            &elements, app, &ctx, &data, &serialized_data, self.id(), &frame.id,
        ).await.with_context(|| format!(
            "Rendering resolved Frame '{}' in Page '{}'", frame.id, self.id()
        ))?;

        if !should_paginate {
            let mut frame_components = Vec::new();
            let mut frame_budget = ComponentBudget::default();
            for element in rendered_elements {
                frame_components.extend(element.components);
                frame_budget = add_budgets(frame_budget, element.budget);
            }
            let budget = add_budgets(reserved, frame_budget);
            validate_message_budget(budget)
                .with_context(|| format!("Validating Page '{}' with Frame '{}'", self.id(), frame.id))?;
            let mut components = prefix_components;
            components.extend(frame_components);
            components.extend(suffix_components);
            return Ok(RenderedPage {
                pages: vec![RenderedMessage { components, attachments: Vec::new(), pagination: None }],
                current_index: 0,
            });
        }

        // Reserve the navigation row before splitting. A Frame whose output fits
        // only without navigation must paginate to make room for those controls.
        let navigation_budget = ComponentBudget {
            total_components: 3, // one Action Row plus two Buttons
            text_display_chars: 0,
            direct_container_children: 0,
        };
        let (planned_pages, page_count) = plan_rendered_frame(
            rendered_elements,
            add_budgets(reserved, navigation_budget),
        ).with_context(|| format!("Paginating Frame '{}' in Page '{}'", frame.id, self.id()))?;

        let mut pages = Vec::with_capacity(page_count);
        for (page_index, (frame_components, budget)) in planned_pages.into_iter().enumerate() {
            validate_message_budget(budget)
                .with_context(|| format!(
                    "Validating Frame '{}' page {} in Page '{}'",
                    frame.id, page_index, self.id()
                ))?;
            let mut components = prefix_components.clone();
            components.extend(frame_components);
            components.extend(suffix_components.clone());

            let pagination = super::frame::pagination::PaginationState {
                frame_id: frame.id.clone(),
                page_index,
                page_count,
            };
            if page_count > 1 {
                components.push(create_pagination_navigation(
                    app, self.id(), &frame.id, &serialized_data, &pagination,
                ).await?);
            }
            pages.push(RenderedMessage {
                components,
                attachments: Vec::new(),
                pagination: (page_count > 1).then_some(pagination),
            });
        }

        Ok(RenderedPage { pages, current_index: 0 })
    }
}
