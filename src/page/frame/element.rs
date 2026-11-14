//! Heterogeneous Frame output and stable developer-defined element keys.
use std::sync::Arc;

use crate::page::component::*;
use crate::page::select::SelectMenu;

/// Stable identity for an interactive element generated inside a Frame.
/// It is based on the Frame ID, developer key, and execution ID, never its current index.
pub struct FrameIdentity {
    pub frame_id: String,
    pub element_key: String,
    pub execution_id: String,
}

/// One heterogeneous component emitted by a Frame, optionally carrying a stable key.
pub struct Element<T, D = Arc<()>> {
    pub component: Component<T, D>,
    pub key: Option<String>,
}

impl<T, D> Element<T, D> {
    /// Creates a Text Display element.
    pub fn text<V: crate::page::value::IntoValue<String, T, D>>(value: V) -> Self {
        Self { component: Component::Text(Text::new(value)), key: None }
    }

    /// Creates a Section element.
    pub fn section(value: Section<T, D>) -> Self {
        Self { component: Component::Section(value), key: None }
    }

    /// Creates a Container element.
    pub fn container(value: Container<T, D>) -> Self {
        Self { component: Component::Container(value), key: None }
    }

    /// Creates a Separator element.
    pub fn separator(value: Separator<T, D>) -> Self {
        Self { component: Component::Separator(value), key: None }
    }

    /// Creates an interactive Button element.
    pub fn button(value: Button<T, D>) -> Self {
        Self { component: Component::Button(value), key: None }
    }

    /// Creates a Button Action Row element.
    pub fn button_row(value: ButtonRow<T, D>) -> Self {
        Self { component: Component::ButtonRow(value), key: None }
    }

    /// Creates a URL Link Button element.
    pub fn link_button(value: LinkButton<T, D>) -> Self {
        Self { component: Component::LinkButton(value), key: None }
    }

    /// Creates a Media Gallery element.
    pub fn gallery(value: MediaGallery<T, D>) -> Self {
        Self { component: Component::MediaGallery(value), key: None }
    }

    /// Creates an attachment-reference element.
    pub fn file(value: File) -> Self {
        Self { component: Component::File(value), key: None }
    }

    /// Creates a select-menu element.
    pub fn selection_menu(value: SelectMenu<T, D>) -> Self {
        Self { component: Component::Select(value), key: None }
    }

    /// Assigns a stable developer-defined key to this Frame element.
    /// Required for interactive elements whose identity must survive pagination.
    pub fn keyed(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }
}
