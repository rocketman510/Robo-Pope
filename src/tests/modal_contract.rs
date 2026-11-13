use serde::Deserialize;
use crate::app::store::derive_custom_id;
use crate::page::{Modal, ModalInput, ModalOption, ModalSubmission, SubmittedValue, TextInputStyle};
use crate::page::modal::input::{ModalSelectInput, TextInput};
use crate::page::value::Value;
use super::support::{app, TestData};

#[tokio::test]
async fn modal_string_select_input_renders() {
    // FIRST: String Select is an advertised Modal input type.
    let app = app();
    let modal = Modal::<TestData>::new("modal-string-select").set_title("Choose")
        .add_select("choice", "Choice", [ModalOption::new("Alpha", "a"), ModalOption::new("Beta", "b")])
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(modal.render(&app, TestData::default()).await.is_ok());
}

#[tokio::test]
async fn modal_checkbox_group_radio_group_and_checkbox_inputs_render() {
    // FIRST: every advertised non-Text Modal input has a working render path.
    let app = app();
    let checkbox = Modal::<TestData>::new("modal-checkbox").set_title("Confirm")
        .add_checkbox("urgent", "Urgent")
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(checkbox.render(&app, TestData::default()).await.is_ok());
    let checks = Modal::<TestData>::new("modal-checkbox-group").set_title("Features")
        .add_checkbox_group("features", "Features", [ModalOption::new("One", "one"), ModalOption::new("Two", "two")])
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(checks.render(&app, TestData::default()).await.is_ok());
    let radio = Modal::<TestData>::new("modal-radio").set_title("Level")
        .add_radio_group("level", "Level", [ModalOption::new("Low", "low"), ModalOption::new("High", "high")])
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(radio.render(&app, TestData::default()).await.is_ok());
}

#[tokio::test]
async fn modal_file_upload_input_renders() {
    // FIRST: File Upload is explicitly listed as a supported Modal input.
    let app = app();
    let modal = Modal::<TestData>::new("modal-file-upload").set_title("Upload")
        .add_file_upload("attachment", "Attachment")
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(modal.render(&app, TestData::default()).await.is_ok());
}

#[tokio::test]
async fn modal_entity_select_inputs_render() {
    // FIRST: User, Role, Mentionable, and Channel selectors are Modal inputs.
    let app = app();
    let variants = [
        ModalInput::UserSelect(ModalSelectInput { id: "users".into(), label: Value::Static("Users".into()), options: vec![] }),
        ModalInput::RoleSelect(ModalSelectInput { id: "roles".into(), label: Value::Static("Roles".into()), options: vec![] }),
        ModalInput::MentionableSelect(ModalSelectInput { id: "mentions".into(), label: Value::Static("Mentions".into()), options: vec![] }),
        ModalInput::ChannelSelect(ModalSelectInput { id: "channels".into(), label: Value::Static("Channels".into()), options: vec![] }),
    ];
    for (index, input) in variants.into_iter().enumerate() {
        let modal = Modal::<TestData>::new(format!("modal-entity-{index}")).set_title("Select")
            .add_input(input)
            .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
            .register(&app).unwrap();
        assert!(modal.render(&app, TestData::default()).await.is_ok());
    }
}

#[tokio::test]
async fn failed_modal_render_does_not_leave_a_persisted_submit_record() {
    // FIRST: failed dynamic resolution must not leave an orphan custom-ID record.
    let app = app();
    let modal = Modal::<TestData>::new("modal-no-orphan").set_title("Failure")
        .add_text_input("field", crate::page::dynamic(
            |_ctx: crate::app::RenderCtx, _data: std::sync::Arc<TestData>| async move {
                anyhow::bail!("dynamic label failed")
            },
        ), TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(modal.render(&app, TestData::default()).await.is_err());
    let data = serde_json::to_vec(&TestData::default()).unwrap();
    let id = derive_custom_id("modal-no-orphan", "submit", &data, None).unwrap();
    assert!(app.store().resolve(&id).await.unwrap().is_none());
}

#[test]
fn modal_user_role_and_channel_values_deserialize_to_discord_id_types() {
    // FIRST: typed deserialization preserves IDs selected by entity menus.
    #[derive(Deserialize)] struct UserForm { users: Vec<serenity::all::UserId> }
    #[derive(Deserialize)] struct RoleForm { roles: Vec<serenity::all::RoleId> }
    #[derive(Deserialize)] struct ChannelForm { channels: Vec<serenity::all::ChannelId> }
    let mut users = ModalSubmission::default();
    users.values.insert("users".into(), SubmittedValue::Users(vec![
        crate::page::SelectedUser { id: serenity::all::UserId::new(123), resolved: None },
    ]));
    assert_eq!(users.deserialize::<UserForm>().unwrap().users, vec![serenity::all::UserId::new(123)]);
    let mut roles = ModalSubmission::default();
    roles.values.insert("roles".into(), SubmittedValue::Roles(vec![
        crate::page::SelectedRole { id: serenity::all::RoleId::new(456), resolved: None },
    ]));
    assert_eq!(roles.deserialize::<RoleForm>().unwrap().roles, vec![serenity::all::RoleId::new(456)]);
    let mut channels = ModalSubmission::default();
    channels.values.insert("channels".into(), SubmittedValue::Channels(vec![
        crate::page::SelectedChannel { id: serenity::all::ChannelId::new(789), resolved: None },
    ]));
    assert_eq!(channels.deserialize::<ChannelForm>().unwrap().channels, vec![serenity::all::ChannelId::new(789)]);
}

#[test]
fn modal_file_upload_values_deserialize_to_attachment_metadata() {
    // FIRST: uploaded file metadata survives typed Serde conversion without downloading bytes.
    #[derive(Deserialize)]
    struct FileMetadata {
        id: serenity::all::AttachmentId, filename: String, url: String, size: u32,
        content_type: Option<String>,
    }
    #[derive(Deserialize)] struct FileForm { files: Vec<FileMetadata> }
    let attachment: serenity::all::Attachment = serde_json::from_value(serde_json::json!({
        "id":"900","filename":"report.txt","title":null,"description":null,"content_type":"text/plain",
        "size":12,"url":"https://cdn.example.test/report.txt","proxy_url":"https://media.example.test/report.txt",
        "height":null,"width":null,"placeholder":null,"placeholder_version":null,"ephemeral":false,
        "duration_secs":null,"waveform":null,"flags":null,"clip_participants":[],"clip_created_at":null,"application":null
    })).unwrap();
    let mut submission = ModalSubmission::default();
    submission.values.insert("files".into(), SubmittedValue::Files(vec![attachment]));
    let form: FileForm = submission.deserialize().unwrap();
    assert_eq!(form.files[0].id, serenity::all::AttachmentId::new(900));
    assert_eq!(form.files[0].filename, "report.txt");
    assert_eq!(form.files[0].size, 12);
    assert_eq!(form.files[0].content_type.as_deref(), Some("text/plain"));
    assert!(form.files[0].url.ends_with("/report.txt"));
}

#[test]
fn modal_mentionable_values_preserve_user_or_role_discriminants() {
    // FIRST: Mentionable values retain their user-versus-role identity.
    #[derive(Deserialize)] struct MentionForm { mentions: Vec<serde_json::Value> }
    let mut submission = ModalSubmission::default();
    submission.values.insert("mentions".into(), SubmittedValue::Mentionables(vec![
        crate::page::SelectedMentionable::User(crate::page::SelectedUser {
            id: serenity::all::UserId::new(123), resolved: None,
        }),
    ]));
    let form: MentionForm = submission.deserialize().unwrap();
    assert_eq!(form.mentions[0]["kind"], "user");
    assert_eq!(form.mentions[0]["id"], 123);
}

#[test]
fn modal_rejects_text_input_maximum_above_discord_limit_at_registration() {
    // FIRST: invalid static Text Input limits are definition errors.
    let app = app();
    let modal = Modal::<TestData>::new("modal-max-length").set_title("Length")
        .add_input(ModalInput::Text(TextInput {
            id: "body".into(), label: Value::Static("Body".into()), style: TextInputStyle::Paragraph,
            required: true, placeholder: None, min_length: None, max_length: Some(4_001),
        }))
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(modal.register(&app).is_err());
}

#[test]
fn modal_rejects_minimum_text_length_greater_than_maximum() {
    // FIRST: impossible input ranges are rejected at registration.
    let app = app();
    let modal = Modal::<TestData>::new("modal-invalid-range").set_title("Range")
        .add_input(ModalInput::Text(TextInput {
            id: "body".into(), label: Value::Static("Body".into()), style: TextInputStyle::Paragraph,
            required: true, placeholder: None, min_length: Some(100), max_length: Some(50),
        }))
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(modal.register(&app).is_err());
}

#[test]
fn modal_rejects_more_than_twenty_five_options_in_every_group_kind() {
    // FIRST: the 25-option Discord limit applies to every Modal option group.
    let app = app();
    let select = Modal::<TestData>::new("modal-too-many-select").set_title("Options")
        .add_select("choice", "Choice", (0..26).map(|n| ModalOption::new(format!("S{n}"), format!("s{n}"))))
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(select.register(&app).is_err());
    let radio = Modal::<TestData>::new("modal-too-many-radio").set_title("Options")
        .add_radio_group("choice", "Choice", (0..26).map(|n| ModalOption::new(format!("R{n}"), format!("r{n}"))))
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(radio.register(&app).is_err());
    let checks = Modal::<TestData>::new("modal-too-many-checkboxes").set_title("Options")
        .add_checkbox_group("choice", "Choice", (0..26).map(|n| ModalOption::new(format!("C{n}"), format!("c{n}"))))
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(checks.register(&app).is_err());
}

#[test]
fn modal_registration_rejects_more_than_five_inputs_and_duplicate_input_ids() {
    // FIRST: Modal shape respects Discord's input count and unique field IDs.
    let app = app();
    let too_many = Modal::<TestData>::new("modal-too-many-inputs").set_title("Too many")
        .add_text_input("a", "A", TextInputStyle::Short).add_text_input("b", "B", TextInputStyle::Short)
        .add_text_input("c", "C", TextInputStyle::Short).add_text_input("d", "D", TextInputStyle::Short)
        .add_text_input("e", "E", TextInputStyle::Short).add_text_input("f", "F", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(too_many.register(&app).is_err());
    let duplicate = Modal::<TestData>::new("modal-duplicate-input").set_title("Duplicate")
        .add_text_input("same", "First", TextInputStyle::Short)
        .add_text_input("same", "Second", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) });
    assert!(duplicate.register(&app).is_err());
}

#[tokio::test]
async fn modal_select_option_description_is_checked_against_discord_limit() {
    // FIRST: resolved option descriptions are checked against Discord's limit.
    let app = app();
    let modal = Modal::<TestData>::new("modal-long-description").set_title("Description")
        .add_select("choice", "Choice", [ModalOption::new("A", "a").set_description("x".repeat(101))])
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    let error = modal.render(&app, TestData::default()).await.err().expect("description validation must fail");
    assert!(format!("{error:#}").contains("100"));
}
