use std::sync::Arc;

use crate::app::store::{derive_custom_id, RoutingRecord, Store};
use crate::page::{Button, ButtonStyle, Modal, ModalSubmission, Page};
use super::support::{app, component_interaction, TestData};

#[tokio::test]
async fn unknown_component_custom_id_returns_a_routing_error() {
    // FIRST: an unknown Discord custom_id is an ordinary routing error and
    // must not dispatch to an arbitrary or default callback.
    let app = app();
    let error = app
        .dispatch_interaction(component_interaction("rp:unknown-custom-id"))
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("custom ID"));
}

#[tokio::test]
async fn stored_data_that_cannot_be_deserialized_returns_a_routing_error() {
    // FIRST: even a valid hash and registered execution must fail cleanly when
    // the stored Page data is malformed.
    let app = app();
    Page::<TestData>::new("bad-stored-data")
        .add_button_row((
            Button::new(
                "Open",
                "open",
                ButtonStyle::Primary,
                None,
                false,
                |_ctx, _data: TestData| async move { Ok(()) },
            ),
            None,
            None,
            None,
            None,
        ))
        .register(&app)
        .unwrap();

    let custom_id = derive_custom_id("bad-stored-data", "open", b"not valid JSON").unwrap();
    app.store()
        .insert(RoutingRecord {
            hash: custom_id.0.clone(),
            definition_id: "bad-stored-data".into(),
            execution_id: "open".into(),
            serialized_data: b"not valid JSON".to_vec(),
        })
        .await
        .unwrap();

    let error = app
        .dispatch_interaction(component_interaction(&custom_id.0))
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("deserial"));
}

#[tokio::test]
async fn component_interaction_cannot_dispatch_a_modal_definition() {
    // FIRST: a Component interaction must not invoke a Modal submit callback,
    // even if a routing record names a valid Modal definition and execution.
    let app = app();
    Modal::<TestData>::new("modal-only-definition")
        .set_title("Modal")
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app)
        .unwrap();

    let serialized = serde_json::to_vec(&TestData::default()).unwrap();
    let custom_id = derive_custom_id(
        "modal-only-definition",
        "submit",
        &serialized,
    )
    .unwrap();

    app.store()
        .insert(RoutingRecord {
            hash: custom_id.0.clone(),
            definition_id: "modal-only-definition".into(),
            execution_id: "submit".into(),
            serialized_data: serialized,
        })
        .await
        .unwrap();

    let error = app
        .dispatch_interaction(component_interaction(&custom_id.0))
        .await
        .unwrap_err();
    assert!(format!("{error:#}").contains("Modal"));
}
