use std::sync::Arc;
use crate::page::{dynamic, resolve, IntoSlot, IntoValue, Slot, Value, ButtonStyle, SeparatorSpacing};
use super::support::{app, TestData};

#[test]
fn static_values_convert_without_a_special_constructor() {
    // First-principles check: the ordinary Rust value is the public input.
    let value: Value<String, TestData> = "hello".into_value();
    assert!(matches!(value, Value::Static(text) if text == "hello"));
}

#[test]
fn supported_primitive_and_enum_values_convert_to_static_values() {
    // First-principles check: every supported property type accepts its natural
    // Rust value, without forcing callers to construct Value::Static manually.
    let enabled: Value<bool, TestData> = true.into_value();
    let color: Value<u32, TestData> = 0x336699.into_value();
    let style: Value<ButtonStyle, TestData> = ButtonStyle::Primary.into_value();
    let spacing: Value<SeparatorSpacing, TestData> = SeparatorSpacing::Large.into_value();
    assert!(matches!(enabled, Value::Static(true)));
    assert!(matches!(color, Value::Static(0x336699)));
    assert!(matches!(style, Value::Static(ButtonStyle::Primary)));
    assert!(matches!(spacing, Value::Static(SeparatorSpacing::Large)));
}

#[tokio::test]
async fn dynamic_values_resolve_with_page_data_and_render_context() {
    // First-principles check: the dynamic-property contract provides both T and
    // App access at render time.
    let app = app();
    let value: Value<String, TestData> = dynamic(
        |ctx: crate::app::RenderCtx<std::sync::Arc<()>>, data: std::sync::Arc<TestData>| async move {
            assert_eq!(ctx.definition_id.as_ref(), "value-test");
            let _store = ctx.app.store();
            Ok(format!("{}:{}", data.label, data.count))
        },
    ).into_value();

    let result = resolve(&value, app.render_context("value-test"), Arc::new(TestData::default()))
        .await
        .unwrap();
    assert_eq!(result, "sample:7");
}

#[tokio::test]
async fn dynamic_value_errors_are_returned_without_being_replaced() {
    // First-principles check: callers receive the original failure, not a
    // fabricated value or an error whose source has been discarded.
    let app = app();
    let value: Value<String, TestData> = dynamic(
        |_ctx, _data| async move {
            Err(anyhow::anyhow!("dynamic lookup failed"))
        },
    ).into_value();

    let error = resolve(&value, app.render_context("error-test"), Arc::new(TestData::default()))
        .await
        .unwrap_err();
    assert_eq!(error.to_string(), "dynamic lookup failed");
}

#[test]
fn optional_slot_converts_none_to_an_empty_slot() {
    // First-principles check: a fixed Section slot can be left empty with normal
    // Option syntax, without exposing the internal Slot type to callers.
    let slot: Slot<String, TestData> = None.into_slot();
    assert!(matches!(slot, Slot::Empty));
}

#[test]
fn optional_slot_converts_static_and_dynamic_values() {
    // First-principles check: fixed slots accept the same static/dynamic choices
    // as ordinary properties.
    let static_slot: Slot<String, TestData> = "Title".into_slot();
    let dynamic_slot: Slot<String, TestData> = dynamic(
        |_ctx: crate::app::RenderCtx<std::sync::Arc<()>>, data: std::sync::Arc<TestData>| async move { Ok(data.label.clone()) },
    ).into_slot();
    assert!(matches!(static_slot, Slot::Value(Value::Static(_))));
    assert!(matches!(dynamic_slot, Slot::Value(Value::Dynamic(_))));
}


#[tokio::test]
async fn direct_async_closure_is_accepted_without_dynamic_adapter() {
    // FIRST: developers can pass asynchronous property resolvers directly.
    use crate::page::Page;
    use std::sync::Arc;
    let app = app();
    let page = Page::<TestData>::new("direct-closure")
        .add_text(|ctx: crate::app::RenderCtx<std::sync::Arc<()>>, data: std::sync::Arc<TestData>| async move {
            assert_eq!(ctx.definition_id.as_ref(), "direct-closure");
            Ok(format!("{}:{}", data.label, data.count))
        }).register(&app).unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();
    assert!(json.to_string().contains("sample:7"));
}

#[tokio::test]
async fn fixed_section_slots_accept_direct_async_closures() {
    // FIRST: direct closures also work in fixed Section slots.
    use crate::page::{Page, Section};
    use std::sync::Arc;
    let app = app();
    let page = Page::<TestData>::new("direct-section-slot")
        .add_section(Section::new().set_text(
            |_ctx: crate::app::RenderCtx<std::sync::Arc<()>>, data: std::sync::Arc<TestData>| async move { Ok(format!("Dynamic {}", data.count)) },
            None, None,
        ).set_accessory(crate::page::Button::new(
            "Open", "section-open", crate::page::ButtonStyle::Primary,
            Some("🔍".into()), false,
            |_ctx, _data: TestData| async move { Ok(()) },
        ))).register(&app).unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();
    assert!(json.to_string().contains("Dynamic 7"));
}

#[tokio::test]
async fn button_properties_accept_direct_async_closures() {
    // FIRST: all display properties share the static-or-dynamic input interface.
    use crate::page::{Button, ButtonStyle, Page};
    use std::sync::Arc;
    let app = app();
    let page = Page::<TestData>::new("direct-button-properties")
        .add_button_row((Button::new(
            |_ctx: crate::app::RenderCtx<std::sync::Arc<()>>, data: std::sync::Arc<TestData>| async move { Ok(format!("Hello {}", data.label)) },
            "dynamic-button",
            |_ctx, _data| async move { Ok(ButtonStyle::Success) },
            |_ctx, _data| async move { Ok(Some("✨".into())) },
            |_ctx, _data| async move { Ok(true) },
            |_ctx, _data: TestData| async move { Ok(()) },
        ), None, None, None, None)).register(&app).unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();
    let output = json.to_string();
    assert!(output.contains("Hello sample"));
    assert!(output.contains("✨"));
    assert!(output.contains("\"disabled\":true"));
}
