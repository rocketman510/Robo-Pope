use crate::page::component::Component;
use crate::page::render::validate_component;
use crate::page::{Button, ButtonStyle, Container, LinkButton, MediaGallery, MediaItem, Page, Section, Text, Thumbnail};
use super::support::app;

#[test]
fn static_text_over_the_limit_is_rejected_during_registration() {
    // FIRST: a static violation is known before rendering and should be rejected
    // while the Page definition is registered.
    let app = app();
    let page = Page::<()>::new("static-text-too-long")
        .add_text("x".repeat(4_001));
    assert!(page.register(&app).is_err());
}

#[tokio::test]
async fn dynamic_text_over_the_limit_is_rejected_after_resolution() {
    // FIRST: the definition can register when its value is not yet known, but
    // the resolved value must be validated before a message is returned.
    let app = app();
    let page = Page::<()>::new("dynamic-text-too-long")
        .add_text(
            |_ctx, _data| async move {
                Ok("x".repeat(4_001))
            },
        )
        .register(&app)
        .unwrap();

    assert!(page.render(&app, ()).await.is_err());
}

#[tokio::test]
async fn forty_message_components_are_allowed() {
    // FIRST: the exact Discord message component limit is valid.
    let app = app();
    let mut page = Page::<()>::new("forty-components");
    for _ in 0..40 {
        page = page.add_text("x");
    }
    let registered = page.register(&app).unwrap();
    let rendered = registered.render(&app, ()).await.unwrap();
    assert_eq!(rendered.components().len(), 40);
}

#[test]
fn forty_one_static_message_components_are_rejected_at_registration() {
    // FIRST: the message-wide component limit is a static structural violation.
    let app = app();
    let mut page = Page::<()>::new("forty-one-components");
    for _ in 0..41 {
        page = page.add_text("x");
    }
    assert!(page.register(&app).is_err());
}

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

#[test]
fn validator_rejects_an_empty_media_gallery() {
    // FIRST: Discord Media Galleries must contain at least one media item.
    let gallery = MediaGallery::<()>::new();
    assert!(validate_component(&Component::MediaGallery(gallery)).is_err());
}

#[test]
fn validator_accepts_a_thumbnail_description_at_1024_characters() {
    // FIRST: a Thumbnail description at the documented boundary is valid.
    let section = Section::<()>::new()
        .set_text("Thumbnail", None, None)
        .set_accessory(
            Thumbnail::new("https://example.test/thumbnail.png")
                .set_description("x".repeat(1_024)),
        );
    assert!(validate_component(&Component::Section(section)).is_ok());
}

#[test]
fn validator_rejects_a_thumbnail_description_over_1024_characters() {
    // FIRST: a Thumbnail description cannot exceed 1024 characters.
    let section = Section::<()>::new()
        .set_text("Thumbnail", None, None)
        .set_accessory(
            Thumbnail::new("https://example.test/thumbnail.png")
                .set_description("x".repeat(1_025)),
        );
    assert!(validate_component(&Component::Section(section)).is_err());
}

#[test]
fn validator_accepts_a_media_gallery_description_at_1024_characters() {
    // FIRST: a Media Gallery item description at the documented boundary is valid.
    let gallery = MediaGallery::<()>::new().add_item(
        MediaItem::new("https://example.test/gallery.png")
            .set_description("x".repeat(1_024)),
    );
    assert!(validate_component(&Component::MediaGallery(gallery)).is_ok());
}

#[test]
fn validator_rejects_a_media_gallery_description_over_1024_characters() {
    // FIRST: a Media Gallery item description cannot exceed 1024 characters.
    let gallery = MediaGallery::<()>::new().add_item(
        MediaItem::new("https://example.test/gallery.png")
            .set_description("x".repeat(1_025)),
    );
    assert!(validate_component(&Component::MediaGallery(gallery)).is_err());
}


#[test]
fn registration_rejects_a_static_button_label_over_eighty_characters() {
    // Page API §13: Discord Button labels are limited to 80 characters.
    // Page API §12: a static violation is rejected during registration.
    let app = app();
    let page = Page::<()>::new("static-button-label-too-long").add_button(
        Button::new(
            "x".repeat(81),
            "open",
            ButtonStyle::Primary,
            None,
            false,
            |_ctx, _data: ()| async move { Ok(()) },
        ),
    );

    assert!(page.register(&app).is_err());
}

#[tokio::test]
async fn rendering_rejects_a_dynamic_button_label_over_eighty_characters() {
    // Page API §§2, 12, and 13: validate a dynamic property after resolution.
    let app = app();
    let page = Page::<()>::new("dynamic-button-label-too-long")
        .add_button(Button::new(
            |_ctx: crate::app::RenderCtx<std::sync::Arc<()>>, _data: std::sync::Arc<()>| async move {
                Ok("x".repeat(81))
            },
            "open",
            ButtonStyle::Primary,
            None,
            false,
            |_ctx, _data: ()| async move { Ok(()) },
        ))
        .register(&app)
        .unwrap();

    assert!(page.render(&app, ()).await.is_err());
}

#[test]
fn registration_rejects_a_static_link_button_label_over_eighty_characters() {
    // Page API §13 applies the Button label limit to Link Buttons as well.
    let app = app();
    let page = Page::<()>::new("static-link-label-too-long").add_link_button(
        LinkButton::new("x".repeat(81), "https://example.test/", None, false),
    );

    assert!(page.register(&app).is_err());
}

#[test]
fn registration_rejects_a_static_link_button_url_over_five_hundred_twelve_characters() {
    // Page API §§12-13: the static Link Button URL limit is known at registration.
    let app = app();
    let url = format!("https://example.test/{}", "x".repeat(500));
    let page = Page::<()>::new("static-link-url-too-long")
        .add_link_button(LinkButton::new("Docs", url, None, false));

    assert!(page.register(&app).is_err());
}

#[tokio::test]
async fn rendering_rejects_a_dynamic_link_button_url_over_five_hundred_twelve_characters() {
    // Page API §§2, 12, and 13: a dynamic Link Button URL is checked after resolution.
    let app = app();
    let page = Page::<()>::new("dynamic-link-url-too-long")
        .add_link_button(LinkButton::new(
            "Docs",
            |_ctx: crate::app::RenderCtx<std::sync::Arc<()>>, _data: std::sync::Arc<()>| async move {
                Ok(format!("https://example.test/{}", "x".repeat(500)))
            },
            None,
            false,
        ))
        .register(&app)
        .unwrap();

    assert!(page.render(&app, ()).await.is_err());
}
