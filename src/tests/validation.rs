use crate::page::component::Component;
use crate::page::render::validate_component;
use crate::page::{Container, MediaGallery, Page, Section, Text};
use super::support::app;

#[test]
fn validator_accepts_a_text_display_at_the_character_limit() {
    // FIRST: a Text Display exactly at Discord's documented limit is valid.
    let component = Component::Text(Text::<()>::new("x".repeat(4_000)));
    assert!(validate_component(&component).is_ok());
}

#[test]
fn validator_rejects_a_text_display_over_the_character_limit() {
    // FIRST: an oversized Text Display returns an error instead of truncating.
    let component = Component::Text(Text::<()>::new("x".repeat(4_001)));
    assert!(validate_component(&component).is_err());
}

#[test]
fn validator_rejects_a_container_with_too_many_direct_children() {
    // FIRST: Container's Discord direct-child limit is enforced structurally.
    let container = (0..11).fold(Container::<()>::new(), |container, index| {
        container.add_text(format!("Child {index}"))
    });
    assert!(validate_component(&Component::Container(container)).is_err());
}

#[test]
fn validator_rejects_a_section_without_any_text_display() {
    // FIRST: a Section must contain at least one Text Display.
    let section = Section::<()>::new();
    assert!(validate_component(&Component::Section(section)).is_err());
}

#[test]
fn validator_rejects_an_empty_button_row_through_the_public_builder() {
    // FIRST: callers use the documented five-slot tuple; an all-empty row is invalid.
    let app = app();
    let page = Page::<()>::new("empty-button-row")
        .add_button_row((None, None, None, None, None));
    assert!(page.register(&app).is_err());
}

#[test]
fn validator_rejects_a_media_gallery_with_more_than_ten_items() {
    // FIRST: the resolved Media Gallery must contain no more than ten items.
    let gallery = (0..11).fold(MediaGallery::<()>::new(), |gallery, index| {
        gallery.add_media(format!("https://example.test/{index}.png"))
    });
    assert!(validate_component(&Component::MediaGallery(gallery)).is_err());
}
