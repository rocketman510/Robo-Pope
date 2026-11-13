//! Declarative Pages, Components V2, Frames, and Modals.
pub mod builder;
pub mod component;
pub mod execution;
pub mod frame;
pub mod limits;
pub mod modal;
pub mod render;
pub mod select;
pub mod value;

pub use builder::{Page,PageDefinition,PageItem,RegisteredPage};
pub use component::{Button,ButtonRow,ButtonSlots,ButtonStyle,Component,Container,
    ContainerItem,File,LinkButton,MediaGallery,MediaItem,Section,SectionAccessory,
    Separator,SeparatorSpacing,Text,Thumbnail};
pub use execution::{Execution,Selection,SelectedUser,SelectedRole,SelectedMentionable,SelectedChannel};
pub use frame::{Element,Frame,FrameIdentity};
pub use modal::{Modal,ModalRender,RegisteredModal};
pub use render::{OutgoingAttachment,RenderedComponent,RenderedMessage,RenderedPage};
pub use select::{ChannelSelect,MentionableSelect,RoleSelect,SelectMenu,SelectOption,StringSelect,UserSelect};
pub use value::{dynamic,resolve,Dynamic,IntoSlot,IntoValue,Slot,Value};
