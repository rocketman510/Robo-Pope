use std::sync::{Arc, atomic::{AtomicBool, Ordering}, Barrier};
use std::thread;
use crate::app::executions::ExecutionKey;
use crate::app::handler::AppHandler;
use crate::app::store::derive_custom_id;
use crate::page::{Button, ButtonStyle, Modal, ModalSubmission, Page, TextInputStyle};
use super::support::{app, component_interaction, modal_submit_interaction, TestData};

#[tokio::test]
async fn page_registration_indexes_each_developer_callback() {
    // FIRST: registration makes every callback available to process-local routing.
    let app = app();
    let _page = Page::<TestData>::new("execution-page")
        .add_button(Button::new("Open", "open-profile", ButtonStyle::Primary,
            |_ctx, _data: TestData| async move { Ok(()) }))
        .register(&app).unwrap();
    assert!(app.executions.contains(&ExecutionKey {
        definition_id: "execution-page".into(), execution_id: "open-profile".into(),
    }));
}

#[tokio::test]
async fn component_dispatch_restores_page_data_and_invokes_the_registered_callback() {
    // FIRST: the full Store -> ExecutionRegistry -> typed callback path must work.
    let app = app();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let data = TestData { label: "restored".into(), count: 42 };
    let serialized = serde_json::to_vec(&data).unwrap();
    let custom_id = derive_custom_id("execution-dispatch", "go", &serialized, None).unwrap();
    let page = Page::<TestData>::new("execution-dispatch")
        .add_button(Button::new("Go", "go", ButtonStyle::Primary,
            move |_ctx, received: TestData| {
                let flag = flag.clone();
                async move {
                    assert_eq!(received.label, "restored");
                    assert_eq!(received.count, 42);
                    flag.store(true, Ordering::SeqCst);
                    Ok(())
                }
            }))
        .register(&app).unwrap();
    page.render(&app, data).await.unwrap();
    app.dispatch_interaction(component_interaction(&custom_id.0)).await.unwrap();
    assert!(called.load(Ordering::SeqCst));
}

fn registered_modal(app: &crate::app::App) -> crate::page::RegisteredModal<TestData> {
    Modal::<TestData>::new("execution-modal").set_title("Submit")
        .add_text_input("name", "Name", TextInputStyle::Short)
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(app).unwrap()
}

#[test]
fn modal_registration_indexes_its_submit_callback() {
    // FIRST: Modal registration inserts its submit closure into ExecutionRegistry.
    let app = app();
    let _modal = registered_modal(&app);
    assert!(app.executions.contains(&ExecutionKey {
        definition_id: "execution-modal".into(), execution_id: "submit".into(),
    }));
}

#[tokio::test]
async fn modal_submit_dispatch_invokes_the_registered_callback() {
    // FIRST: a submitted Modal custom ID resolves and invokes the typed submit callback.
    let app = app();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let modal = Modal::<TestData>::new("modal-dispatch").set_title("Submit")
        .add_text_input("name", "Name", TextInputStyle::Short)
        .on_submit(move |_ctx, data, _submission: ModalSubmission| {
            let flag = flag.clone();
            async move {
                assert_eq!(data.count, 7);
                flag.store(true, Ordering::SeqCst);
                Ok(())
            }
        }).register(&app).unwrap();
    let rendered = modal.render(&app, TestData::default()).await.unwrap();
    app.dispatch_interaction(modal_submit_interaction(&rendered.custom_id)).await.unwrap();
    assert!(called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn gateway_handler_forwards_interaction_create_to_app_dispatch() {
    // FIRST: the gateway adapter must hand incoming interactions to the App router.
    let app = app();
    let called = Arc::new(AtomicBool::new(false));
    let flag = called.clone();
    let data = TestData::default();
    let serialized = serde_json::to_vec(&data).unwrap();
    let custom_id = derive_custom_id("handler-dispatch", "click", &serialized, None).unwrap();
    let page = Page::<TestData>::new("handler-dispatch")
        .add_button(Button::new("Click", "click", ButtonStyle::Secondary,
            move |_ctx, _data: TestData| {
                let flag = flag.clone();
                async move { flag.store(true, Ordering::SeqCst); Ok(()) }
            }))
        .register(&app).unwrap();
    page.render(&app, data).await.unwrap();
    AppHandler::new(app).dispatch_interaction(component_interaction(&custom_id.0)).await.unwrap();
    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn page_registration_rejects_id_already_owned_by_modal() {
    // FIRST: Page and Modal definitions share one application-wide ID namespace.
    let app = app();
    Modal::<TestData>::new("shared-page-first").set_title("Modal")
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).unwrap();
    assert!(Page::<TestData>::new("shared-page-first").add_text("Page").register(&app).is_err());
}

#[test]
fn modal_registration_rejects_id_already_owned_by_page() {
    // FIRST: uniqueness works in the reverse registration order as well.
    let app = app();
    Page::<TestData>::new("shared-modal-first").add_text("Page").register(&app).unwrap();
    assert!(Modal::<TestData>::new("shared-modal-first").set_title("Modal")
        .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
        .register(&app).is_err());
}

#[test]
fn page_and_modal_id_uniqueness_is_atomic_under_concurrent_registration() {
    // FIRST: Page and Modal registration must reserve their shared ID atomically.
    let app = app();
    let page_app = app.clone();
    let modal_app = app.clone();
    let barrier = Arc::new(Barrier::new(3));
    let pbarrier = barrier.clone();
    let mbarrier = barrier.clone();
    let page_thread = thread::spawn(move || {
        pbarrier.wait();
        Page::<TestData>::new("concurrent-shared-id").add_text("Page").register(&page_app).is_ok()
    });
    let modal_thread = thread::spawn(move || {
        mbarrier.wait();
        Modal::<TestData>::new("concurrent-shared-id").set_title("Modal")
            .on_submit(|_ctx, _data, _submission: ModalSubmission| async move { Ok(()) })
            .register(&modal_app).is_ok()
    });
    barrier.wait();
    let page_won = page_thread.join().unwrap();
    let modal_won = modal_thread.join().unwrap();
    assert_ne!(page_won, modal_won, "exactly one definition type may reserve the ID");
}
