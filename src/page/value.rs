use std::sync::Arc;
use crate::app::RenderCtx;
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
    // First-principles check: properties accept their natural Rust value, not a
    // manually constructed Value::Static wrapper.
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
    // First-principles check: dynamic properties receive the shared Page data and
    // App-linked context at render time.
    let app = app();
    let value: Value<String, TestData> = dynamic(
        |ctx: RenderCtx, data: Arc<TestData>| async move {
            assert_eq!(ctx.definition_id.as_ref(), "value-test");
            let _db = ctx.app.db.clone();
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
        |_ctx: RenderCtx, _data: Arc<TestData>| async move {
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
    // First-principles check: an unused fixed Section slot can be written as
    // ordinary None instead of exposing internal Slot or Value constructors.
    let slot: Slot<String, TestData> = None.into_slot();
    assert!(matches!(slot, Slot::Empty));
}

#[test]
fn optional_slot_converts_static_and_dynamic_values() {
    // First-principles check: fixed slots accept the same static/dynamic choices
    // as ordinary component properties.
    let static_slot: Slot<String, TestData> = "Title".into_slot();
    let dynamic_slot: Slot<String, TestData> = dynamic(
        |_ctx: RenderCtx, data: Arc<TestData>| async move { Ok(data.label.clone()) },
    ).into_slot();
    assert!(matches!(static_slot, Slot::Value(Value::Static(_))));
    assert!(matches!(dynamic_slot, Slot::Value(Value::Dynamic(_))));
}
