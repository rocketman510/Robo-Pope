use crate::page::{Execution, Selection};
use super::support::TestData;

#[test]
fn button_execution_has_a_stable_developer_defined_id() {
    // First-principles check: the ID is the lookup key for an execution within
    // a registered definition, so this tests a real developer-facing guarantee.
    let execution = Execution::<TestData>::new("approve-user", |_ctx, data| async move {
        let _ = data.count;
        Ok(())
    });
    assert_eq!(execution.id, "approve-user");
}

#[test]
fn typed_execution_accepts_an_interaction_payload_separate_from_page_data() {
    // First-principles check: Page data and interaction input are distinct values.
    // This checks the callback shape directly rather than testing router internals.
    let execution = Execution::<TestData, Vec<String>>::with_input(
        "choose-roles",
        |_ctx, data, selected| async move {
            let _page_value = data.label;
            let _selected_values = selected;
            Ok(())
        },
    );
    assert_eq!(execution.id, "choose-roles");
}

#[test]
fn selection_keeps_each_supported_payload_kind_distinct() {
    // First-principles check: downstream callbacks need typed values rather than
    // an unlabelled Vec<String> for every Discord select menu.
    let selection = Selection::Strings(vec!["alpha".into(), "beta".into()]);
    assert!(matches!(selection, Selection::Strings(values) if values.len() == 2));
}
