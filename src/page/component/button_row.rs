//! An Action Row represented by exactly five Button slots.
use super::button::Button;

/// Exactly five optional Button slots, matching Discord's Action Row capacity.
pub type ButtonSlots<T> = (
    Option<Button<T>>,
    Option<Button<T>>,
    Option<Button<T>>,
    Option<Button<T>>,
    Option<Button<T>>,
);

/// A Button Action Row. At least one and at most five slots must be populated.
pub struct ButtonRow<T> {
    /// The five possible Button positions in their rendered order.
    pub slots: ButtonSlots<T>,
}
impl<T> ButtonRow<T> {
    /// Creates a row from its five fixed positions.
    pub fn new(slots: ButtonSlots<T>) -> Self { Self { slots } }
    /// Counts populated slots; render validation rejects an empty row.
    pub fn populated(&self) -> usize {
        todo!("count populated slots; rendering validates one to five")
    }
}
