//! Rendered output is the framework/Serenity boundary.
use std::sync::Arc;
use serenity::builder::CreateComponent;
use super::frame::pagination::PaginationState;

/// One Serenity component produced by the render layer.
#[derive(Clone)]
pub struct RenderedComponent {
    pub(crate) component:CreateComponent<'static>,
}
/// File bytes to attach to the outgoing Discord message.
/// File components refer to attachments by filename; they do not contain the bytes.
pub struct OutgoingAttachment {
    pub filename:String,
    pub bytes:Arc<Vec<u8>>,
}
/// Complete Components V2 message payload plus its attachments and pagination metadata.
pub struct RenderedMessage {
    pub(crate) components:Vec<RenderedComponent>,
    pub(crate) attachments:Vec<OutgoingAttachment>,
    pub(crate) pagination:Option<PaginationState>,
}
/// All rendered message pages for one Page instance and its current position.
pub struct RenderedPage {
    pub pages:Vec<RenderedMessage>,
    pub current_index:usize,
}
impl RenderedMessage {
    /// Borrows the rendered component tree.
    pub fn components(&self)->&[RenderedComponent] { &self.components }
    /// Borrows outgoing attachment bytes and filenames.
    pub fn attachments(&self)->&[OutgoingAttachment] { &self.attachments }
    /// Returns pagination metadata when this message represents a paginated Frame.
    pub fn pagination(&self)->Option<&PaginationState> { self.pagination.as_ref() }
    /// Adds outgoing file bytes. A File component can reference the same filename.
    pub fn add_attachment(mut self,filename:impl Into<String>,bytes:Vec<u8>)->Self {
        self.attachments.push(OutgoingAttachment { filename:filename.into(),bytes:Arc::new(bytes) });
        self
    }
    /// Consumes the rendered message without losing attachment or pagination data.
    ///
    /// The returned tuple contains Serenity components, the outgoing file
    /// attachments, and the optional pagination metadata, in that order.
    pub fn into_parts(
        self,
    ) -> (
        Vec<CreateComponent<'static>>,
        Vec<OutgoingAttachment>,
        Option<PaginationState>,
    ) {
        let components = self.components
            .into_iter()
            .map(|component| component.component)
            .collect();

        (components, self.attachments, self.pagination)
    }

    /// Extracts only the Serenity components.
    ///
    /// This intentionally discards attachments and pagination metadata. Use
    /// `into_parts` when the complete rendered payload must be sent or retained.
    pub fn into_components(self) -> Vec<CreateComponent<'static>> {
        self.into_parts().0
    }
}

/// Validates a resolved component tree against the Discord component and text budgets.
/// Validation occurs after dynamic properties and Frame contents have been expanded.
pub(crate) fn validate_component<T>(
    _component:&super::component::Component<T>
)->anyhow::Result<()> {
    use super::component::{Component, ContainerItem, SectionAccessory};
    use super::value::{Slot, Value};

    fn text_chars<T>(value: &super::value::Value<String, T>) -> usize {
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
                    SectionAccessory::Thumbnail(_) | SectionAccessory::Button(_) => {}
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
                gallery.items.len() <= super::limits::MEDIA_GALLERY_MAX_ITEMS,
                "Media Gallery has {} items; maximum is {}",
                gallery.items.len(),
                super::limits::MEDIA_GALLERY_MAX_ITEMS
            );
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
