use std::collections::HashMap;
use serde::Deserialize;
use crate::page::{
    Modal, ModalInput, ModalOption, ModalSubmission, SubmittedValue, TextInputStyle,
};
use super::support::{app, TestData};

#[test]
fn modal_builder_starts_with_its_developer_defined_id_and_title() {
    // First-principles check: the ID locates the Modal definition and title is
    // directly visible to the user. No internal rendering structure is inspected.
    let modal = Modal::<TestData>::new("edit-user").set_title("Edit User");
    assert_eq!(modal.id, "edit-user");
    assert_eq!(modal.title, "Edit User");
}

#[test]
fn modal_input_ids_are_the_keys_used_for_submitted_values() {
    // First-principles check: developers need stable input identifiers to read
    // fields after submission. This exercises the public add-input builder.
    let modal = Modal::<TestData>::new("ticket")
        .add_text_input("subject", "Subject", TextInputStyle::Short);
    assert!(matches!(&modal.inputs[0], ModalInput::Text(input) if input.id == "subject"));
}

#[test]
fn modal_raw_submit_callback_receives_context_data_and_submission_separately() {
    // First-principles check: submitted form values are not the Modal's original
    // serialized data. This test proves the intended callback parameter shape.
    let modal = Modal::<TestData>::new("edit-user").on_submit(|ctx, data, submission| async move {
        let _db = ctx.app.db.clone();
        let _original_user = data.label;
        let _form_name = submission.text("username")?;
        Ok(())
    });
    assert!(modal.submit.is_some());
}

#[test]
fn modal_typed_submit_callback_uses_serde_form_data() {
    // First-principles check: the typed callback should receive the developer's
    // Rust form type, avoiding manual field parsing in application code.
    #[derive(Deserialize)]
    struct EditForm { username: String }

    let modal = Modal::<TestData>::new("edit-user")
        .on_submit_typed(|_ctx, _data, form: EditForm| async move {
            let _username = form.username;
            Ok(())
        });
    assert!(modal.submit.is_some());
}

#[test]
fn modal_registration_returns_a_typed_registered_handle() {
    // First-principles check: this tests the registration boundary and returned
    // typestate instead of manually constructing a RegisteredModal.
    let app = app();
    let registered = Modal::<TestData>::new("ticket")
        .set_title("Create Ticket")
        .on_submit(|_ctx, _data, _submission| async move { Ok(()) })
        .register(&app)
        .unwrap();
    assert_eq!(registered.definition.id, "ticket");
}

#[test]
fn submission_text_accessor_returns_the_value_for_its_input_id() {
    // First-principles check: callers retrieve a submitted field by the stable
    // ID they assigned to the corresponding Modal input.
    let submission = ModalSubmission {
        values: HashMap::from([("subject".into(), SubmittedValue::Text("Help".into()))]),
    };
    assert_eq!(submission.text("subject").unwrap(), "Help");
}

#[test]
fn submission_accessor_rejects_missing_and_wrong_type_values() {
    // First-principles check: invalid or absent form values must be errors, never
    // invented defaults or silent coercions.
    let submission = ModalSubmission {
        values: HashMap::from([("roles".into(), SubmittedValue::Strings(vec!["admin".into()]))]),
    };
    assert!(submission.text("missing").is_err());
    assert!(submission.text("roles").is_err());
}

#[test]
fn typed_submission_deserializes_by_input_id() {
    // First-principles check: Serde maps form IDs to struct fields. This verifies
    // the promised ergonomic alternative to hand-written extraction.
    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct CreateTicket { subject: String, urgent: bool }

    let submission = ModalSubmission {
        values: HashMap::from([
            ("subject".into(), SubmittedValue::Text("Access request".into())),
            ("urgent".into(), SubmittedValue::Boolean(true)),
        ]),
    };
    let form: CreateTicket = submission.deserialize().unwrap();
    assert_eq!(form, CreateTicket { subject: "Access request".into(), urgent: true });
}

#[test]
fn modal_options_keep_label_and_submitted_value_independent() {
    // First-principles check: a displayed option label is presentation; the value
    // returned to application code must remain a separate stable string.
    let option = ModalOption::<TestData>::new("Staff Member", "staff");
    assert_eq!(option.value, "staff");
}
