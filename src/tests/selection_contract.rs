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
    let error = page.render(&app, TestData::default()).await.err().expect("unsupported option state must fail");
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
                |_ctx, _data| async move { Ok(true) },
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

#[tokio::test]
async fn string_select_renders_placeholder_min_max_and_dynamic_option_properties() {
    // FIRST: message Select menus render resolved option labels/descriptions and
    // preserve their submitted values and selection bounds.

    let app = app();
    let page = Page::<TestData>::new("string-select-bounds")
        .add_selection_menu(
            StringSelect::new(
                "Pick up to two",
                "select-bounds",
                [
                    SelectOption::new(
                        |_ctx, data| async move {
                            Ok(format!("{} role", data.label))
                        },
                        "member",
                    )
                    .set_description(
                        |_ctx, data| async move {
                            Ok(format!("Level {}", data.count))
                        },
                    ),
                ],
                |_ctx, _data: TestData, _values: Vec<String>| async move { Ok(()) },
            )
            .set_min_values(1)
            .set_max_values(2),
        )
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();
    assert!(json.contains("Pick up to two"));
    assert!(json.contains("sample role"));
    assert!(json.contains("Level 7"));
    assert!(json.contains("member"));
    assert!(json.contains("\\"min_values\\":1"));
    assert!(json.contains("\\"max_values\\":2"));
}

#[tokio::test]
async fn message_string_select_rejects_static_option_description_over_one_hundred_characters() {
    // FIRST: the static message-select description limit is rejected before
    // an invalid menu is sent.
    let app = app();
    let page = Page::<TestData>::new("long-select-description")
        .add_selection_menu(StringSelect::new(
            "Choose",
            "choice",
            [SelectOption::new("Option", "option").set_description("x".repeat(101))],
            |_ctx, _data: TestData, _values: Vec<String>| async move { Ok(()) },
        ))
        .register(&app)
        .unwrap();

    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn message_string_select_rejects_dynamic_option_description_over_one_hundred_characters() {
    // FIRST: a dynamic property is validated after it resolves just like a
    // static property, without being silently truncated.
    let app = app();
    let page = Page::<TestData>::new("dynamic-long-select-description")
        .add_selection_menu(StringSelect::new(
            "Choose",
            "choice",
            [SelectOption::new("Option", "option").set_description(
                |_ctx, _data| async move {
                    Ok("x".repeat(101))
                },
            )],
            |_ctx, _data: TestData, _values: Vec<String>| async move { Ok(()) },
        ))
        .register(&app)
        .unwrap();

    assert!(page.render(&app, TestData::default()).await.is_err());
}

#[tokio::test]
async fn all_entity_select_kinds_render_their_discord_component_types() {
    // FIRST: User, Role, Mentionable, and Channel Selects have distinct Discord
    // component kinds and preserve their own resolved placeholder and bounds.
    use crate::page::{ChannelSelect, MentionableSelect, RoleSelect, UserSelect};

    fn find_component_type(value: &serde_json::Value, expected: u64) -> Option<&serde_json::Value> {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("type").and_then(serde_json::Value::as_u64) == Some(expected) {
                    return Some(value);
                }
                map.values().find_map(|child| find_component_type(child, expected))
            }
            serde_json::Value::Array(items) => items.iter().find_map(|child| find_component_type(child, expected)),
            _ => None,
        }
    }

    let app = app();
    let page = Page::<TestData>::new("all-entity-selects")
        .add_selection_menu(
            UserSelect::new("users", |_ctx, _data: TestData, _values: Vec<crate::page::SelectedUser>| async move { Ok(()) })
                .set_placeholder("Choose users")
                .set_min_values(1)
                .set_max_values(3),
        )
        .add_selection_menu(
            RoleSelect::new("roles", |_ctx, _data: TestData, _values: Vec<crate::page::SelectedRole>| async move { Ok(()) })
                .set_placeholder("Choose roles")
                .set_min_values(1)
                .set_max_values(2),
        )
        .add_selection_menu(
            MentionableSelect::new("mentions", |_ctx, _data: TestData, _values: Vec<crate::page::SelectedMentionable>| async move { Ok(()) })
                .set_placeholder("Choose mentions")
                .set_min_values(1)
                .set_max_values(2),
        )
        .add_selection_menu(
            ChannelSelect::new("channels", |_ctx, _data: TestData, _values: Vec<crate::page::SelectedChannel>| async move { Ok(()) })
                .set_placeholder("Choose channels")
                .set_min_values(1)
                .set_max_values(4),
        )
        .register(&app)
        .unwrap();

    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap();

    for (component_type, placeholder, max_values) in [
        (5, "Choose users", 3),
        (6, "Choose roles", 2),
        (7, "Choose mentions", 2),
        (8, "Choose channels", 4),
    ] {
        let component = find_component_type(&json, component_type)
            .unwrap_or_else(|| panic!("missing entity Select type {component_type}"));
        assert_eq!(component["placeholder"], placeholder);
        assert_eq!(component["min_values"], 1);
        assert_eq!(component["max_values"], max_values);
    }
}

#[tokio::test]
async fn page_user_select_renders_with_placeholder() {
    // FIRST: a Page-level User Select is a supported message component.
    use crate::page::UserSelect;
    let app = app();
    let page = Page::<TestData>::new("page-user-select")
        .add_selection_menu(
            UserSelect::new(
                "user-select",
                |_ctx, _data: TestData, _users: Vec<crate::page::SelectedUser>| async move { Ok(()) },
            )
            .set_placeholder("Pick a user"),
        )
        .register(&app)
        .unwrap();
    let rendered = page.render(&app, TestData::default()).await.unwrap();
    let json = serde_json::to_value(rendered.into_components()).unwrap().to_string();
    assert!(json.contains("Pick a user"));
    assert!(json.contains("\\"type\\":5"));
}
