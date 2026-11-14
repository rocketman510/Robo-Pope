//! Rendered output is the framework/Serenity boundary.
use std::sync::Arc;
use serenity::builder::CreateComponent;

#[derive(Clone)]
pub struct RenderedComponent {
    pub(crate) component: CreateComponent<'static>,
}

pub struct OutgoingAttachment {
    pub filename: String,
    pub bytes: Arc<Vec<u8>>,
}

/// One complete Components V2 message and its outgoing attachments.
pub struct RenderedMessage {
    pub(crate) components: Vec<RenderedComponent>,
    pub(crate) attachments: Vec<OutgoingAttachment>,
}
impl std::fmt::Debug for RenderedMessage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("RenderedMessage")
            .field("component_count", &self.components.len())
            .field("attachment_count", &self.attachments.len())
            .finish()
    }
}
impl RenderedMessage {
    pub fn components(&self) -> &[RenderedComponent] { &self.components }
    pub fn attachments(&self) -> &[OutgoingAttachment] { &self.attachments }
    pub fn add_attachment(mut self, filename: impl Into<String>, bytes: Vec<u8>) -> Self {
        self.attachments.push(OutgoingAttachment {
            filename: filename.into(), bytes: Arc::new(bytes),
        });
        self
    }
    /// Consumes the rendered message into Serenity components and attachments.
    /// The empty trailing unit preserves compatibility with existing tuple destructuring.
    pub fn into_parts(self) -> (Vec<CreateComponent<'static>>, Vec<OutgoingAttachment>, ()) {
        let components = self.components.into_iter().map(|part| part.component).collect();
        (components, self.attachments, ())
    }
    pub fn into_components(self) -> Vec<CreateComponent<'static>> {
        self.components.into_iter().map(|part| part.component).collect()
    }
}

/// Validates a resolved component tree against the Discord component and text budgets.
/// Validation occurs after dynamic properties and Frame contents have been expanded.
pub(crate) fn validate_component<T, D>(
    _component:&super::component::Component<T, D>
)->anyhow::Result<()> {
    use super::component::{Component, ContainerItem, SectionAccessory};
    use super::value::{Slot, Value};

    fn text_chars<T, D>(value: &super::value::Value<String, T, D>) -> usize {
        match value {
            Value::Static(text) => text.chars().count(),
            Value::Dynamic(_) => 0, // The resolved value is checked during rendering.
        }
    }

    match _component {
        Component::Text(text) => {
            anyhow::ensure!(
                text_chars(&text.content) <= super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS,
                "Text Display exceeds {} characters",
                super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS
            );
        }
        Component::Section(section) => {
            let slots = [&section.text.0, &section.text.1, &section.text.2];
            let populated = slots.iter().filter(|slot| matches!(slot, Slot::Value(_))).count();
            anyhow::ensure!(
                populated >= super::limits::SECTION_MIN_TEXT_DISPLAYS,
                "Section must contain at least one Text Display"
            );
            anyhow::ensure!(
                populated <= super::limits::SECTION_MAX_TEXT_DISPLAYS,
                "Section contains too many Text Displays"
            );
            anyhow::ensure!(
                section.accessory.is_some(),
                "Section requires a Button or Thumbnail accessory"
            );
            let chars = slots.iter().map(|slot| match slot {
                Slot::Empty => 0,
                Slot::Value(value) => text_chars(value),
            }).sum::<usize>();
            anyhow::ensure!(
                chars <= super::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS,
                "Section Text Displays exceed the message text budget"
            );
            if let Some(accessory) = &section.accessory {
                match accessory {
                    SectionAccessory::Thumbnail(thumbnail) => {
                        if let Some(Value::Static(description)) = &thumbnail.description {
                            anyhow::ensure!(
                                description.chars().count() <= 1_024,
                                "Thumbnail description exceeds 1024 characters"
                            );
                        }
                    }
                    SectionAccessory::Button(_) => {}
                }
            }
        }
        Component::Container(container) => {
            anyhow::ensure!(
                container.children.len() <= super::limits::CONTAINER_MAX_DIRECT_CHILDREN,
                "Container has {} direct children; maximum is {}",
                container.children.len(),
                super::limits::CONTAINER_MAX_DIRECT_CHILDREN
            );
            for child in &container.children {
                if let ContainerItem::Component(component) = child {
                    validate_component(component)?;
                }
            }
        }
        Component::ButtonRow(row) => {
            anyhow::ensure!(
                (super::limits::BUTTON_ROW_MIN_BUTTONS..=super::limits::BUTTON_ROW_MAX_BUTTONS)
                    .contains(&row.populated()),
                "Button Action Row must contain between {} and {} buttons",
                super::limits::BUTTON_ROW_MIN_BUTTONS,
                super::limits::BUTTON_ROW_MAX_BUTTONS
            );
        }
        Component::MediaGallery(gallery) => {
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
            for item in &gallery.items {
                if let Some(Value::Static(description)) = &item.description {
                    anyhow::ensure!(
                        description.chars().count() <= 1_024,
                        "Media Gallery item description exceeds 1024 characters"
                    );
                }
            }
        }
        Component::Select(crate::page::select::SelectMenu::String(menu)) => {
            anyhow::ensure!(
                menu.options.len() <= super::limits::STRING_SELECT_MAX_OPTIONS,
                "String Select has too many options"
            );
        }
        Component::Select(_) => {}
        Component::Separator(_)
        | Component::File(_)
        | Component::Button(_)
        | Component::LinkButton(_) => {}
    }
    Ok(())
}
