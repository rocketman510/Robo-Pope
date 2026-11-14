use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::app::RenderCtx;
use crate::page::{Container, Element, Page, Section, Separator};
use super::support::{app, TestData};

#[tokio::test]
async fn frame_resolver_receives_render_context_and_shared_page_data() {
    // FIRST: Frame callbacks receive the same App-linked rendering context and
    // generic Page data used by ordinary dynamic properties.
    let app = app();
    let page = Page::<TestData>::new("frame-context")
        .add_frame(|ctx: RenderCtx, data: Arc<TestData>| async move {
            let _database = ctx.app.db.clone();
            Ok(vec![Element::text(format!("{}:{}", data.label, data.count))])
        })
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();
    assert!(json.to_string().contains("sample:7"));
}

#[tokio::test]
async fn frame_expands_heterogeneous_elements_in_returned_order() {
    // FIRST: a Frame returns a heterogeneous collection and the Page preserves
    // that collection's order at the Frame's declared position.
    let app = app();
    let page = Page::<TestData>::new("frame-heterogeneous")
        .add_frame(|_ctx, _data| async move {
            Ok(vec![
                Element::text("First element"),
                Element::section(Section::new().set_text("Second element", None, None)),
                Element::container(
                    Container::new()
                        .add_text("Third element")
                        .add_separator(Separator::new()),
                ),
            ])
        })
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();
    let first = json.find("First element").unwrap();
    let second = json.find("Second element").unwrap();
    let third = json.find("Third element").unwrap();
    assert!(first < second && second < third);
}

#[tokio::test]
async fn frame_resolver_is_invoked_once_per_page_render() {
    // FIRST: a Frame callback is evaluated once when its Page is rendered.
    let app = app();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();

    let page = Page::<TestData>::new("frame-once")
        .add_frame(move |_ctx, _data| {
            let observed = observed.clone();
            async move {
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(vec![Element::text("Resolved once")])
            }
        })
        .register(&app)
        .unwrap();

    page.render(&app, TestData::default()).await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn multiple_frames_expand_in_page_order() {
    // FIRST: a Page may contain multiple ordinary Frames. Each expands at its
    // declared position, and no Frame needs a separate named identity.
    let app = app();
    let page = Page::<TestData>::new("multiple-frames")
        .add_text("Header")
        .add_frame(|_ctx, _data| async move {
            Ok(vec![Element::text("First Frame")])
        })
        .add_separator(Separator::new())
        .add_frame(|_ctx, _data| async move {
            Ok(vec![Element::text("Second Frame")])
        })
        .add_text("Footer")
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();
    let header = json.find("Header").unwrap();
    let first = json.find("First Frame").unwrap();
    let second = json.find("Second Frame").unwrap();
    let footer = json.find("Footer").unwrap();
    assert!(header < first && first < second && second < footer);
}

#[tokio::test]
async fn frame_expansion_that_exceeds_message_component_limit_returns_an_error() {
    // FIRST: Frame does not silently paginate or truncate a collection that
    // makes the complete Page exceed Discord's message component limit.
    let app = app();
    let page = Page::<TestData>::new("frame-overflow")
        .add_frame(|_ctx, _data| async move {
            Ok((0..41)
                .map(|index| Element::text(format!("Element {index}")))
                .collect())
        })
        .register(&app)
        .unwrap();

    let error = page.render(&app, TestData::default()).await.unwrap_err();
    assert!(format!("{error:#}").contains("frame-overflow"));
}

#[tokio::test]
async fn an_empty_frame_contributes_no_components() {
    // FIRST: a Frame callback may return an empty collection.
    let app = app();
    let page = Page::<TestData>::new("empty-frame")
        .add_frame(|_ctx, _data| async move {
            Ok(Vec::<Element<TestData>>::new())
        })
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    assert!(rendered.components().is_empty());
}
