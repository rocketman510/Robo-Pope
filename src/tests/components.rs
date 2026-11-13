use crate::page::{
    Button, ButtonRow, ButtonSlots, ButtonStyle, Component, Container, ContainerItem,
    File, LinkButton, MediaGallery, Section, SectionAccessory, Separator,
    Slot, Text, Thumbnail,
};
use super::support::TestData;

#[test]
fn section_supports_three_independent_text_slots_and_empty_slots() {
    // First-principles check: a Section exposes exactly the fixed three slots Discord supports.
    let section = Section::<TestData>::new().set_text("Heading", None, "Details");
    assert!(matches!(&section.text.0, Slot::Value(_)));
    assert!(matches!(&section.text.1, Slot::Empty));
    assert!(matches!(&section.text.2, Slot::Value(_)));
}

#[test]
fn section_accepts_a_thumbnail_as_its_single_accessory() {
    // First-principles check: this tests the documented composition path, not Serenity internals.
    let thumbnail = Thumbnail::<TestData>::new("https://example.test/image.png").set_description("Preview");
    let section = Section::<TestData>::new()
        .set_text("Details", None, None)
        .set_accessory(thumbnail);
    assert!(matches!(section.accessory, Some(SectionAccessory::Thumbnail(_))));
}

#[test]
fn button_row_has_five_explicit_optional_slots() {
    // First-principles check: the tuple itself prevents a sixth slot from being supplied.
    let slots: ButtonSlots<TestData> = (None, None, None, None, None);
    let row = ButtonRow::new(slots);
    assert_eq!(row.populated(), 0);
}

#[test]
fn container_preserves_the_declared_order_of_mixed_children() {
    // First-principles check: visible child order is a public guarantee.
    let container = Container::<TestData>::new()
        .add_text("First")
        .add_separator(Separator::new())
        .add_text("Last");
    assert_eq!(container.children.len(), 3);
    assert!(matches!(&container.children[0], ContainerItem::Component(Component::Text(_))));
    assert!(matches!(&container.children[1], ContainerItem::Component(Component::Separator(_))));
    assert!(matches!(&container.children[2], ContainerItem::Component(Component::Text(_))));
}

#[test]
fn media_gallery_keeps_items_in_the_order_added() {
    // First-principles check: order determines the visible gallery sequence.
    let gallery = MediaGallery::<TestData>::new()
        .add_media("https://example.test/one.png")
        .add_media("https://example.test/two.png");
    assert_eq!(gallery.items.len(), 2);
    match &gallery.items[0].url {
        crate::page::Value::Static(url) => assert_eq!(url, "https://example.test/one.png"),
        crate::page::Value::Dynamic(_) => panic!("the supplied URL was static"),
    }
    match &gallery.items[1].url {
        crate::page::Value::Static(url) => assert_eq!(url, "https://example.test/two.png"),
        crate::page::Value::Dynamic(_) => panic!("the supplied URL was static"),
    }
}

#[test]
fn button_callback_receives_page_data_and_app_linked_context() {
    // First-principles check: this verifies the callback shape developers actually use.
    let button = Button::<TestData>::new(
        "Open",
        "open-user",
        ButtonStyle::Primary,
        |ctx, data| async move {
            let _db = ctx.app.db.clone();
            let _label = data.label;
            Ok(())
        },
    );
    assert_eq!(button.id, "open-user");
}

#[test]
fn link_button_is_configured_with_a_url_without_an_execution() {
    // First-principles check: URL navigation is distinct from a callback execution.
    let link = LinkButton::<TestData>::new("Documentation", "https://example.test/docs");
    assert!(matches!(link.label, crate::page::Value::Static(_)));
    assert!(matches!(link.url, crate::page::Value::Static(_)));
}

#[test]
fn file_component_references_an_outgoing_attachment_by_filename() {
    // First-principles check: File references an attachment; bytes are attached separately.
    let file = File::new("report.pdf");
    assert_eq!(file.filename, "report.pdf");
}

#[test]
fn text_display_accepts_a_dynamic_content_value() {
    // First-principles check: static and dynamic text share one constructor.
    let text = Text::<TestData>::new(crate::page::dynamic(
        |_ctx: crate::app::RenderCtx, data: std::sync::Arc<TestData>| async move {
            Ok(data.label.clone())
        },
    ));
    assert!(matches!(text.content, crate::page::Value::Dynamic(_)));
}

#[test]
fn separator_defaults_to_small_spacing_and_visible_divider() {
    // First-principles check: these are observable defaults, independent of rendering.
    let separator = Separator::<TestData>::new();
    assert!(matches!(separator.spacing, crate::page::Value::Static(crate::page::SeparatorSpacing::Small)));
    assert!(matches!(separator.divider, crate::page::Value::Static(true)));
}
