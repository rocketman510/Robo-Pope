//! Section: one to three independent Text Display slots plus one accessory.
use crate::page::value::{IntoSlot, Slot};
use super::button::Button;
use super::thumbnail::Thumbnail;

pub enum SectionAccessory<T, E> {
    Thumbnail(Thumbnail<T, E>),
    Button(Button<T, E>),
}

pub struct Section<T, E> {
    pub text: (Slot<String, T, E>, Slot<String, T, E>, Slot<String, T, E>),
    pub accessory: Option<SectionAccessory<T, E>>,
}
impl<T, E> Section<T, E> {
    pub fn new() -> Self {
        Self { text:(Slot::Empty, Slot::Empty, Slot::Empty), accessory:None }
    }

    pub fn set_text<A, B, C>(mut self, first:A, second:B, third:C) -> Self
    where
        A: IntoSlot<String, T, E>,
        B: IntoSlot<String, T, E>,
        C: IntoSlot<String, T, E>,
    {
        self.text = (first.into_slot(), second.into_slot(), third.into_slot());
        self
    }

    pub fn set_accessory(mut self, accessory:impl Into<SectionAccessory<T, E>>) -> Self {
        self.accessory = Some(accessory.into());
        self
    }
}
impl<T,E> From<Thumbnail<T,E>> for SectionAccessory<T,E> {
    fn from(value:Thumbnail<T,E>)->Self { Self::Thumbnail(value) }
}
impl<T,E> From<Button<T,E>> for SectionAccessory<T,E> {
    fn from(value:Button<T,E>)->Self { Self::Button(value) }
}
