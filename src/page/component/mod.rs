//! Components V2 building blocks.
//!
//! Components are framework-owned descriptions that resolve into Serenity
//! builders only inside rendering. The public API models the concepts
//! developers compose, not every Serenity builder method.
//!
//! Component<T, E> is the common abstraction used by Page, Container, and
//! Frame Element. It supports heterogeneous Frame results without manual
//! Serenity conversion in application code.
//!
//! Container children are restricted to component forms Discord permits.
//! Thumbnail is legal as a Section accessory, not as a standalone component.
//! Interactive identity collection must recurse through Section accessories
//! and nested Containers.
//!
//! Intended common representation (illustrative):
//! \`\`\`rust,ignore
//! enum Component<T, E> {
//!     Text(Text<T, E>),
//!     Section(Section<T, E>),
//!     Container(Container<T, E>),
//!     Separator(Separator<T, E>),
//!     MediaGallery(MediaGallery<T, E>),
//!     File(File),
//!     Button(Button<T, E>),
//!     LinkButton(LinkButton<T, E>),
//!     Select(SelectMenu<T, E>),
//!     ButtonRow(ButtonRow<T, E>),
//! }
//! \`\`\`
//!
//! TODO: Define Component<T, E> and resolve/validate contracts.
//! TODO: Keep Serenity builders out of public application-facing APIs.
//! TODO: Centralize recursive identity collection and component accounting.
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
