use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use crate::{
    app::{RenderCtx, store::{CustomId, DefinitionKind, Store, derive_custom_id}},
    page::{
        Button, ButtonStyle, Element, Frame, Modal, Page, TextInputStyle, dynamic,
    },
};
use super::support::{app, TestData};

#[tokio::test]
async fn page_render_resolves_dynamic_values_with_page_data_and_app_context() {
    let app = app();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();

    let page = Page::<TestData>::new("render-dynamic")
        .add_text(dynamic(
            move |ctx: RenderCtx, data: Arc<TestData>| {
                let observed = observed.clone();
                async move {
                    assert_eq!(ctx.definition_id.as_ref(), "render-dynamic");
                    observed.fetch_add(1, Ordering::SeqCst);
                    Ok(format!("{}:{}", data.label, data.count))
                }
            },
        ))
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(rendered.components().len(), 1);
}

#[tokio::test]
async fn page_render_preserves_dynamic_callback_errors_and_adds_page_context() {
    let app = app();
    let page = Page::<TestData>::new("render-error")
        .add_text(dynamic(
            |_ctx: RenderCtx, _data: Arc<TestData>| async move {
                Err(anyhow::anyhow!("the dynamic source failed"))
            },
        ))
        .register(&app)
        .unwrap();

    let error = page.render(&app, TestData::default()).await.unwrap_err();
    let chain = format!("{error:#}");

    assert!(chain.contains("the dynamic source failed"));
    assert!(chain.contains("render-error"));
}

#[tokio::test]
async fn page_render_validates_the_combined_text_display_budget_after_resolution() {
    let app = app();
    let page = Page::<TestData>::new("render-text-budget")
        .add_text("a".repeat(2_500))
        .add_text("b".repeat(1_501))
        .register(&app)
        .unwrap();

    let error = page.render(&app, TestData::default()).await.unwrap_err();
    assert!(format!("{error:#}").contains("render-text-budget"));
}

#[tokio::test]
async fn page_render_persists_button_execution_identity_and_page_data() {
    let app = app();
    let data = TestData::default();
    let serialized_data = serde_json::to_vec(&data).unwrap();
    let expected_id = derive_custom_id(
        "render-persist",
        "open-profile",
        &serialized_data,
        None,
    )
    .unwrap();

    let page = Page::<TestData>::new("render-persist")
        .add_button(Button::new(
            "Open profile",
            "open-profile",
            ButtonStyle::Primary,
            |_ctx, _data: TestData| async move { Ok(()) },
        ))
        .register(&app)
        .unwrap();

    let _rendered = page.render(&app, data).await.unwrap();
    let record = app.store()
        .resolve(&expected_id)
        .await
        .unwrap()
        .expect("render must persist its interactive execution");

    assert_eq!(record.definition_kind, DefinitionKind::Page);
    assert_eq!(record.definition_id, "render-persist");
    assert_eq!(record.execution_id, "open-profile");
    assert_eq!(record.serialized_data, serialized_data);
}

#[tokio::test]
async fn render_pages_keeps_static_content_and_splits_frame_elements_in_order() {
    let app = app();
    let frame = Frame::<TestData>::new(
        "user-rows",
        |_ctx: RenderCtx, data: Arc<TestData>| async move {
            Ok((0..41)
                .map(|index| {
                    Element::text(format!("{}-{index}", data.label))
                        .keyed(format!("user-{index}"))
                })
                .collect())
        },
    );

    let page = Page::<TestData>::new("render-paginated")
        .add_text("Static header")
        .add_frame(frame)
        .add_text("Static footer")
        .paginate()
        .register(&app)
        .unwrap();

    let rendered = page.render_pages(&app, TestData::default()).await.unwrap();

    assert_eq!(rendered.current_index, 0);
    assert_eq!(rendered.pages.len(), 2);
    assert!(rendered.pages.iter().all(|page| page.components().len() <= 40));
    assert_eq!(rendered.pages[0].pagination().unwrap().page_index, 0);
    assert_eq!(rendered.pages[1].pagination().unwrap().page_index, 1);
    assert_eq!(rendered.pages[0].pagination().unwrap().page_count, 2);
}

#[tokio::test]
async fn modal_render_persists_modal_data_and_returns_a_modal_response() {
    let app = app();
    let data = TestData::default();
    let serialized_data = serde_json::to_vec(&data).unwrap();
    let expected_id = derive_custom_id("render-modal", "submit", &serialized_data, None).unwrap();

    let modal = Modal::<TestData>::new("render-modal")
        .set_title("Edit profile")
        .add_text_input("username", "Username", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission| async move { Ok(()) })
        .register(&app)
        .unwrap();

    let rendered = modal.render(&app, data).await.unwrap();

    assert_eq!(rendered.custom_id, expected_id.0);
    let record = app.store()
        .resolve(&CustomId(rendered.custom_id.clone()))
        .await
        .unwrap()
        .expect("rendering a Modal must persist its submit execution");
    assert_eq!(record.definition_kind, DefinitionKind::Modal);
    assert_eq!(record.definition_id, "render-modal");
    assert_eq!(record.execution_id, "submit");
    assert_eq!(record.serialized_data, serialized_data);
}

#[test]
fn page_registration_rejects_duplicate_execution_ids() {
    let app = app();
    let page = Page::<TestData>::new("duplicate-executions")
        .add_button(Button::new(
            "First",
            "duplicate",
            ButtonStyle::Primary,
            |_ctx, _data: TestData| async move { Ok(()) },
        ))
        .add_button(Button::new(
            "Second",
            "duplicate",
            ButtonStyle::Secondary,
            |_ctx, _data: TestData| async move { Ok(()) },
        ));

    assert!(page.register(&app).is_err());
}

#[test]
fn page_registration_rejects_duplicate_frame_ids() {
    let app = app();
    let first = Frame::<TestData>::new("duplicate-frame", |_ctx, _data| async move {
        Ok(Vec::<Element<TestData>>::new())
    });
    let second = Frame::<TestData>::new("duplicate-frame", |_ctx, _data| async move {
        Ok(Vec::<Element<TestData>>::new())
    });

    let page = Page::<TestData>::new("duplicate-frames")
        .add_frame(first)
        .add_frame(second);

    assert!(page.register(&app).is_err());
}
#[test]
fn rendered_message_parts_conversion_preserves_attachments_and_pagination() {
    use crate::page::frame::pagination::PaginationState;
    use crate::page::{OutgoingAttachment, RenderedMessage};

    let message = RenderedMessage {
        components: Vec::new(),
        attachments: vec![OutgoingAttachment {
            filename: "report.pdf".into(),
            bytes: Arc::new(vec![1, 2, 3]),
        }],
        pagination: Some(PaginationState {
            frame_id: "users".into(),
            page_index: 1,
            page_count: 3,
        }),
    };

    let (components, attachments, pagination) = message.into_parts();

    assert!(components.is_empty());
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].filename, "report.pdf");
    assert_eq!(attachments[0].bytes.as_slice(), &[1, 2, 3]);
    assert_eq!(pagination.unwrap().page_index, 1);
}
