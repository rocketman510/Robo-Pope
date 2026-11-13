//! Framework-owned Components V2 descriptions. Serenity builders are created
//! only by the render layer; Thumbnail is a Section accessory, not standalone.
pub mod button;
pub mod button_row;
pub mod container;
pub mod file;
pub mod link_button;
pub mod media_gallery;
pub mod section;
pub mod separator;
pub mod text;
pub mod thumbnail;

pub use button::{Button,ButtonStyle};
pub use button_row::{ButtonRow,ButtonSlots};
pub use container::{Container,ContainerItem};
pub use file::File;
pub use link_button::LinkButton;
pub use media_gallery::{MediaGallery,MediaItem};
pub use section::{Section,SectionAccessory};
pub use separator::{Separator,SeparatorSpacing};
pub use text::Text;
pub use thumbnail::Thumbnail;

use crate::page::select::SelectMenu;
pub enum Component<T> {
    Text(Text<T>),
    Section(Section<T>),
    Container(Container<T>),
    Separator(Separator<T>),
    MediaGallery(MediaGallery<T>),
    File(File),
    Button(Button<T>),
    LinkButton(LinkButton<T>),
    Select(SelectMenu<T>),
    ButtonRow(ButtonRow<T>),
}
