use crate::page::{Page, SelectOption, StringSelect};
use super::support::{app, TestData};

#[tokio::test]
async fn disabled_string_select_option_is_rejected_instead_of_silently_ignored() {
    // FIRST: Discord has no per-option disabled field for String Select.
    let app = app();
    let page = Page::<TestData>::new("disabled-option")
        .add_selection_menu(StringSelect::new(
            "Choose", "choice",
            [SelectOption::new("Enabled", "enabled"), SelectOption::new("Disabled", "disabled").set_disabled(true)],
            |_ctx, _data: TestData, _values: Vec<String>| async move { Ok(()) },
        )).register(&app).unwrap();
    let error = page.render(&app, TestData::default()).await.unwrap_err();
    assert!(format!("{error:#}").contains("do not support per-option disabled"));
}

#[tokio::test]
async fn dynamically_disabled_string_select_option_is_rejected_after_resolution() {
    // FIRST: dynamic properties obey the same unsupported-state rule.
    let app = app();
    let page = Page::<TestData>::new("dynamic-disabled-option")
        .add_selection_menu(StringSelect::new(
            "Choose", "choice",
            [SelectOption::new("Disabled", "disabled").set_disabled(
                |_ctx: crate::app::RenderCtx, _data: std::sync::Arc<TestData>| async move { Ok(true) },
            )],
            |_ctx, _data: TestData, _values: Vec<String>| async move { Ok(()) },
        )).register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn disabling_the_entire_select_menu_remains_supported() {
    // FIRST: rejecting a disabled option does not reject the supported menu state.
    let app = app();
    let page = Page::<TestData>::new("disabled-menu")
        .add_selection_menu(StringSelect::new(
            "Choose", "choice", [SelectOption::new("One", "one")],
            |_ctx, _data: TestData, _values: Vec<String>| async move { Ok(()) },
        ).set_disabled(true)).register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_ok());
}
