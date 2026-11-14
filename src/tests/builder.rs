use crate::page::{Component, Page, PageItem, Separator};
use super::support::{app, TestData};

#[test]
fn page_builder_preserves_top_level_component_order() {
    // First-principles check: developers compose a message in order. Inspecting
    // the resulting order is the smallest direct proof of that public behavior.
    let page = Page::<TestData>::new("ordered-page")
        .add_text("Title")
        .add_separator(Separator::new())
        .add_text("Footer");

    assert_eq!(page.id(), "ordered-page");
    assert_eq!(page.items.len(), 3);
    assert!(matches!(&page.items[0], PageItem::Component(Component::Text(_))));
    assert!(matches!(&page.items[1], PageItem::Component(Component::Separator(_))));
    assert!(matches!(&page.items[2], PageItem::Component(Component::Text(_))));
}

#[test]
fn page_data_type_is_shared_by_static_and_dynamic_components() {
    // First-principles check: both properties belong to the same Page data scope.
    // The closure reads T and the App-linked context without an extra generic.
    let page = Page::<TestData>::new("shared-data")
        .add_text("Static")
        .add_text(|ctx: crate::app::RenderCtx<std::sync::Arc<()>>, data: std::sync::Arc<TestData>| async move {
            let _db = ctx.app.db.clone();
            Ok(format!("{} {}", data.label, data.count))
        });
    assert_eq!(page.items.len(), 2);
    assert!(matches!(&page.items[1], PageItem::Component(Component::Text(text))
        if matches!(text.content, crate::page::Value::Dynamic(_))));
}

#[test]
fn registering_a_page_returns_an_immutable_registered_handle() {
    // First-principles check: registration is the typestate boundary promised by
    // the API. This must exercise register rather than simulate the handle.
    let app = app();
    let registered = Page::<TestData>::new("register-me")
        .add_text("Hello")
        .register(&app)
        .unwrap();
    assert_eq!(registered.id(), "register-me");
}
