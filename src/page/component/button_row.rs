//! An Action Row represented by exactly five Button slots.
use super::button::Button;
use super::link_button::LinkButton;

/// One position in the five-slot Button Action Row.
pub enum ButtonSlot<T, D = std::sync::Arc<()>> {
    Button(Button<T, D>),
    LinkButton(LinkButton<T, D>),
    Empty,
}

impl<T, D> From<Button<T, D>> for ButtonSlot<T, D> {
    fn from(value: Button<T, D>) -> Self { Self::Button(value) }
}

impl<T, D> From<LinkButton<T, D>> for ButtonSlot<T, D> {
    fn from(value: LinkButton<T, D>) -> Self { Self::LinkButton(value) }
}

impl<T, D> From<Option<Button<T, D>>> for ButtonSlot<T, D> {
    fn from(value: Option<Button<T, D>>) -> Self {
        value.map(Self::Button).unwrap_or(Self::Empty)
    }
}

/// Exactly five positions, each accepting a Button, LinkButton, or None.
pub type ButtonSlots<T, D = std::sync::Arc<()>> = (
    ButtonSlot<T, D>,
    ButtonSlot<T, D>,
    ButtonSlot<T, D>,
    ButtonSlot<T, D>,
    ButtonSlot<T, D>,
);

/// A Button Action Row. At least one and at most five slots must be populated.
pub struct ButtonRow<T, D = std::sync::Arc<()>> {
    /// The five possible Button positions in their rendered order.
    pub slots: ButtonSlots<T, D>,
}

impl<T, D> ButtonRow<T, D> {
    /// Creates a row from its five fixed positions.
    pub fn new(slots: ButtonSlots<T, D>) -> Self { Self { slots } }

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
