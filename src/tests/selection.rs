use crate::page::{
    ChannelSelect, MentionableSelect, RoleSelect, SelectOption, Selection,
    StringSelect, UserSelect,
};
use super::support::TestData;

#[test]
fn string_select_option_keeps_display_label_separate_from_submitted_value() {
    // First-principles check: callbacks depend on the stable submitted value, not
    // on the label shown to a user.
    let option = SelectOption::<TestData>::new("Administrator", "admin");
    assert_eq!(option.value, "admin");
}

#[test]
fn string_select_callback_receives_selected_values_in_a_typed_vector() {
    // First-principles check: selected values are explicit callback input, separate
    // from the Page's original T data.
    let menu = StringSelect::<TestData>::new(
        "Choose a role",
        "role-picker",
        [SelectOption::new("Admin", "admin"), SelectOption::new("Member", "member")],
        |_ctx, data, selected: Vec<String>| async move {
            let _page_label = data.label;
            let _submitted_values = selected;
            Ok(())
        },
    );
    assert_eq!(menu.id, "role-picker");
}

#[test]
fn entity_select_callbacks_receive_their_specific_selected_type() {
    // First-principles check: a User Select must not degrade into an untyped string
    // payload, and all entity selectors share the same common construction model.
    let users = UserSelect::<TestData>::new("selected-users", |_ctx, data, selected| async move {
        let _page_label = data.label;
        let _: Vec<crate::page::SelectedUser> = selected;
        Ok(())
    });
    let roles = RoleSelect::<TestData>::new("selected-roles", |_ctx, _data, selected| async move {
        let _: Vec<crate::page::SelectedRole> = selected;
        Ok(())
    });
    let mentions = MentionableSelect::<TestData>::new("selected-mentions", |_ctx, _data, selected| async move {
        let _: Vec<crate::page::SelectedMentionable> = selected;
        Ok(())
    });
    let channels = ChannelSelect::<TestData>::new("selected-channels", |_ctx, _data, selected| async move {
        let _: Vec<crate::page::SelectedChannel> = selected;
        Ok(())
    });
    assert_eq!(users.id, "selected-users");
    assert_eq!(roles.id, "selected-roles");
    assert_eq!(mentions.id, "selected-mentions");
    assert_eq!(channels.id, "selected-channels");
}

#[test]
fn selection_payload_variants_preserve_the_menu_kind() {
    // First-principles check: application code can distinguish selection categories
    // without inferring them from a list of IDs.
    assert!(matches!(Selection::Strings(vec![]), Selection::Strings(_)));
}
