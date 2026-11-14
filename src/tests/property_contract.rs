use std::sync::Arc;

use crate::app::RenderCtx;
use crate::page::{
    Button, ButtonStyle, Container, LinkButton, MediaGallery, MediaItem, Page,
    Section, SelectOption, Separator, SeparatorSpacing, StringSelect, Thumbnail,
};
use super::support::{app, TestData};

#[tokio::test]
async fn link_button_label_url_emoji_and_disabled_state_resolve_dynamically() {
    // FIRST: every display property on a Link Button accepts a direct async
    // callback, including optional emoji and the disabled Boolean.
    let app = app();
    let page = Page::<TestData>::new("dynamic-link-properties")
        .add_button_row((
            LinkButton::new(
                |_ctx: RenderCtx, data: Arc<TestData>| async move {
                    Ok(format!("{} docs", data.label))
                },
                |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                    Ok("https://example.test/dynamic".to_owned())
                },
                |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                    Ok(Some("🌐".to_owned()))
                },
                |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                    Ok(true)
                },
            ),
            None,
            None,
            None,
            None,
        ))
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();

    assert!(json.contains("sample docs"));
    assert!(json.contains("https://example.test/dynamic"));
    assert!(json.contains("🌐"));
    assert!(json.contains("\"disabled\":true"));
}

#[tokio::test]
async fn dynamic_container_color_separator_thumbnail_and_gallery_properties_resolve() {
    // FIRST: non-interactive presentation properties resolve after the Page
    // is built, including color, separator settings, image metadata, and media.
    let app = app();
    let page = Page::<TestData>::new("dynamic-presentation-properties")
        .add_container(
            Container::new()
                .set_accent_color(
                    |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                        Ok(0x345678_u32)
                    },
                )
                .add_text("Container")
        )
        .add_separator(
            Separator::new()
                .set_spacing(
                    |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                        Ok(SeparatorSpacing::Large)
                    },
                )
                .set_divider(
                    |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                        Ok(false)
                    },
                )
        )
        .add_section(
            Section::new()
                .set_text("Preview", None, None)
                .set_accessory(
                    Thumbnail::new(
                        |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                            Ok("https://example.test/dynamic-thumbnail.png".to_owned())
                        },
                    )
                    .set_description(
                        |_ctx: RenderCtx, data: Arc<TestData>| async move {
                            Ok(format!("{} thumbnail", data.label))
                        },
                    )
                    .set_spoiler(
                        |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                            Ok(true)
                        },
                    )
                )
        )
        .add_gallery(
            MediaGallery::new().set_media([
                MediaItem::new(
                    |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                        Ok("https://example.test/dynamic-gallery.png".to_owned())
                    },
                )
                .set_description(
                    |_ctx: RenderCtx, data: Arc<TestData>| async move {
                        Ok(format!("{} gallery image", data.label))
                    },
                )
                .set_spoiler(
                    |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                        Ok(true)
                    },
                ),
            ])
        )
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();

    assert!(json.contains("Container"));
    assert!(json.contains("3430008"), "dynamic accent color must reach the payload");
    assert!(json.contains("dynamic-thumbnail.png"));
    assert!(json.contains("sample thumbnail"));
    assert!(json.contains("dynamic-gallery.png"));
    assert!(json.contains("sample gallery image"));
    assert!(json.contains("\"spoiler\":true"));
}

#[tokio::test]
async fn string_select_placeholder_option_label_and_description_resolve_dynamically() {
    // FIRST: menu placeholders and option display text support direct dynamic
    // callbacks, while submitted option values remain static developer values.
    let app = app();
    let page = Page::<TestData>::new("dynamic-select-properties")
        .add_selection_menu(
            StringSelect::new(
                |_ctx: RenderCtx, data: Arc<TestData>| async move {
                    Ok(format!("Choose for {}", data.label))
                },
                "choose",
                [
                    SelectOption::new(
                        |_ctx: RenderCtx, data: Arc<TestData>| async move {
                            Ok(format!("Option {}", data.count))
                        },
                        "option-value",
                    )
                    .set_description(
                        |_ctx: RenderCtx, data: Arc<TestData>| async move {
                            Ok(format!("Description {}", data.label))
                        },
                    )
                    .set_emoji(
                        |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                            Ok(Some("✨".to_owned()))
                        },
                    ),
                ],
                |_ctx, _data: TestData, _selected: Vec<String>| async move { Ok(()) },
            )
            .set_min_values(1)
            .set_max_values(1)
        )
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();

    assert!(json.contains("Choose for sample"));
    assert!(json.contains("Option 7"));
    assert!(json.contains("Description sample"));
    assert!(json.contains("✨"));
    assert!(json.contains("option-value"));
    assert!(json.contains("\"min_values\":1"));
    assert!(json.contains("\"max_values\":1"));
}
