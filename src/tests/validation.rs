use crate::page::component::Component;
use crate::page::render::validate_component;
use crate::page::{ButtonRow, ButtonSlots, Container, MediaGallery, Section, Text};

#[test]
fn validator_accepts_a_text_display_at_the_character_limit() {
    // First-principles check: values exactly at a documented limit remain valid.
    let component = Component::Text(Text::<()>::new("x".repeat(4_000)));
    assert!(validate_component(&component).is_ok());
}

#[test]
fn validator_rejects_a_text_display_over_the_character_limit() {
    // First-principles check: an oversized component must error, not truncate text.
    let component = Component::Text(Text::<()>::new("x".repeat(4_001)));
    assert!(validate_component(&component).is_err());
}

#[test]
fn validator_rejects_a_container_with_too_many_direct_children() {
    // First-principles check: Discord's direct-child limit is a structural rule
    // that should be enforced without a live Discord interaction.
    let container = (0..11).fold(Container::<()>::new(), |container, index| {
        container.add_text(format!("Child {index}"))
    });
    assert!(validate_component(&Component::Container(container)).is_err());
}

#[test]
fn validator_rejects_a_section_without_any_text_display() {
    // First-principles check: a Section must contain at least one Text Display.
    let section = Section::<()>::new();
    assert!(validate_component(&Component::Section(section)).is_err());
}

#[test]
fn validator_rejects_an_empty_button_row() {
    // First-principles check: the tuple has five possible slots, but Discord still
    // requires at least one actual Button.
    let slots: ButtonSlots<()> = (None, None, None, None, None);
    let row = ButtonRow::new(slots);
    assert!(validate_component(&Component::ButtonRow(row)).is_err());
}

#[test]
fn validator_rejects_a_media_gallery_with_more_than_ten_items() {
    // First-principles check: validation must enforce the documented Gallery cap.
    let gallery = (0..11).fold(MediaGallery::<()>::new(), |gallery, index| {
        gallery.add_media(format!("https://example.test/{index}.png"))
    });
    assert!(validate_component(&Component::MediaGallery(gallery)).is_err());
}
