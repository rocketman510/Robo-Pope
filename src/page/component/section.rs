//! Section: one to three independent Text Display slots plus one accessory.
use crate::page::value::{IntoSlot, Slot};
use super::button::Button;
use super::thumbnail::Thumbnail;

/// The single accessory allowed beside a Section's Text Display slots.
pub enum SectionAccessory<T, D = std::sync::Arc<()>> {
    Thumbnail(Thumbnail<T, D>),
    Button(Button<T, D>),
}

/// Section with up to three Text Display slots and at most one Button or Thumbnail accessory.
/// At least one text slot must be populated before rendering.
pub struct Section<T, D = std::sync::Arc<()>> {
    /// Three fixed Text Display slots; at least one must be populated.
    pub text: (Slot<String, T, D>, Slot<String, T, D>, Slot<String, T, D>),
    /// Optional Button or Thumbnail accessory. Only one is allowed.
    pub accessory: Option<SectionAccessory<T, D>>,
}
impl<T, D> Section<T, D> {
    /// Starts a Section with all text slots empty and no accessory.
    pub fn new() -> Self {
        Self { text:(Slot::Empty, Slot::Empty, Slot::Empty), accessory:None }
    }

    /// Replaces all three text slots. Each slot accepts a static value, a dynamic
    /// value, or an empty `Option`; at least one must be populated for rendering.
    pub fn set_text<A, B, C>(mut self, first:A, second:B, third:C) -> Self
    where
        A: IntoSlot<String, T, D>,
        B: IntoSlot<String, T, D>,
        C: IntoSlot<String, T, D>,
    {
        self.text = (first.into_slot(), second.into_slot(), third.into_slot());
        self
    }

    /// Sets the Section's one accessory, replacing any accessory already assigned.
    pub fn set_accessory(mut self, accessory:impl Into<SectionAccessory<T, D>>) -> Self {
        self.accessory = Some(accessory.into());
        self
    }
}
impl<T> From<Thumbnail<T, D>> for SectionAccessory<T, D> {
    fn from(value:Thumbnail<T, D>)->Self { Self::Thumbnail(value) }
}
impl<T> From<Button<T, D>> for SectionAccessory<T, D> {
    fn from(value:Button<T, D>)->Self { Self::Button(value) }
}
