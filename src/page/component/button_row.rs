//! An Action Row represented by exactly five Button slots.
use super::button::Button;

pub type ButtonSlots<T> = (
    Option<Button<T>>,
    Option<Button<T>>,
    Option<Button<T>>,
    Option<Button<T>>,
    Option<Button<T>>,
);

pub struct ButtonRow<T> {
    pub slots: ButtonSlots<T>,
}
impl<T> ButtonRow<T> {
    pub fn new(slots: ButtonSlots<T>) -> Self { Self { slots } }
    pub fn populated(&self) -> usize {
        todo!("count populated slots; rendering validates one to five")
    }
}
