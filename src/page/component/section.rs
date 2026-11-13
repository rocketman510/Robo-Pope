//! Section: one to three independent Text Display slots plus one accessory.
use crate::page::value::{IntoSlot, Slot};
use super::button::Button;
use super::thumbnail::Thumbnail;

pub enum SectionAccessory<T> {
    Thumbnail(Thumbnail<T>),
    Button(Button<T>),
}

pub struct Section<T> {
    pub text: (Slot<String, T>, Slot<String, T>, Slot<String, T>),
    pub accessory: Option<SectionAccessory<T>>,
}
impl<T> Section<T> {
    pub fn new() -> Self {
        Self { text:(Slot::Empty, Slot::Empty, Slot::Empty), accessory:None }
    }

    pub fn set_text<A, B, C>(mut self, first:A, second:B, third:C) -> Self
    where
        A: IntoSlot<String, T>,
        B: IntoSlot<String, T>,
        C: IntoSlot<String, T>,
    {
        self.text = (first.into_slot(), second.into_slot(), third.into_slot());
        self
    }

    pub fn set_accessory(mut self, accessory:impl Into<SectionAccessory<T>>) -> Self {
        self.accessory = Some(accessory.into());
        self
    }
}
impl<T> From<Thumbnail<T>> for SectionAccessory<T> {
    fn from(value:Thumbnail<T>)->Self { Self::Thumbnail(value) }
}
impl<T> From<Button<T>> for SectionAccessory<T> {
    fn from(value:Button<T>)->Self { Self::Button(value) }
}
