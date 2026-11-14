//! An Action Row represented by exactly five Button slots.
use super::button::Button;
use super::link_button::LinkButton;

/// One position in the five-slot Button Action Row.
pub enum ButtonSlot<T> {
    Button(Button<T>),
    LinkButton(LinkButton<T>),
    Empty,
}

impl<T> From<Button<T>> for ButtonSlot<T> {
    fn from(value: Button<T>) -> Self { Self::Button(value) }
}

impl<T> From<LinkButton<T>> for ButtonSlot<T> {
    fn from(value: LinkButton<T>) -> Self { Self::LinkButton(value) }
}

impl<T> From<Option<Button<T>>> for ButtonSlot<T> {
    fn from(value: Option<Button<T>>) -> Self {
        value.map(Self::Button).unwrap_or(Self::Empty)
    }
}

/// Exactly five positions, each accepting a Button, LinkButton, or None.
pub type ButtonSlots<T> = (
    ButtonSlot<T>,
    ButtonSlot<T>,
    ButtonSlot<T>,
    ButtonSlot<T>,
    ButtonSlot<T>,
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
        [
            &self.slots.0,
            &self.slots.1,
            &self.slots.2,
            &self.slots.3,
            &self.slots.4,
        ]
        .into_iter()
        .filter(|slot| !matches!(slot, ButtonSlot::Empty))
        .count()
    }
}
