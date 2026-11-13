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
pub enum Component<T,E> {
    Text(Text<T,E>),
    Section(Section<T,E>),
    Container(Container<T,E>),
    Separator(Separator<T,E>),
    MediaGallery(MediaGallery<T,E>),
    File(File),
    Button(Button<T,E>),
    LinkButton(LinkButton<T,E>),
    Select(SelectMenu<T,E>),
    ButtonRow(ButtonRow<T,E>),
}
