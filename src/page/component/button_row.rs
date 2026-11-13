//! An Action Row represented by exactly five Button slots.
use super::button::Button;

pub type ButtonSlots<T, E> = (
    Option<Button<T, E>>,
    Option<Button<T, E>>,
    Option<Button<T, E>>,
    Option<Button<T, E>>,
    Option<Button<T, E>>,
);

pub struct ButtonRow<T, E> {
    pub slots: ButtonSlots<T, E>,
}
impl<T, E> ButtonRow<T, E> {
    pub fn new(slots: ButtonSlots<T, E>) -> Self { Self { slots } }
    pub fn populated(&self) -> usize {
        todo!("count populated slots; rendering validates one to five")
    }
}
