//! Focused legacy contracts that are not part of the T-01 through T-15 audit matrix.
use crate::page::{Page, UserSelect};
use super::support::{app, TestData};

#[tokio::test]
async fn page_entity_user_select_menu_renders() {
    // FIRST: Page-level entity Select menus are independent from Modal entity inputs.
    let app = app();
    let page = Page::<TestData>::new("page-user-select")
        .add_selection_menu(UserSelect::new(
            "user-select",
            |_ctx, _data: TestData, _users| async move { Ok(()) },
        ).set_placeholder("Pick a user"))
        .register(&app).unwrap();
    assert!(page.render(&app, TestData::default()).await.is_ok());
}
