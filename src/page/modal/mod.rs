//! Modal form blueprint, independent of message Pages.
pub mod input;
pub mod submission;

pub use input::{CheckboxGroupInput,CheckboxInput,FileUploadInput,ModalInput,ModalOption,
    ModalSelectInput,RadioGroupInput,TextInput,TextInputStyle};
pub use submission::{ModalSubmission,SubmittedValue};

use std::future::Future;
use std::sync::Arc;
use serde::{Serialize,de::DeserializeOwned};
use crate::app::{App,Ctx,Result};
use crate::page::execution::Execution;
use crate::page::render::RenderedMessage;
use crate::page::value::IntoValue;

/// Mutable builder for a Discord Modal form.
/// The Modal's `T` is serialized when opened and reconstructed on submission.
pub struct Modal<T> {
    pub(crate) id:String,
    pub(crate) title:String,
    pub(crate) inputs:Vec<ModalInput<T>>,
    pub(crate) submit:Option<Execution<T,ModalSubmission>>,
}
/// Immutable Modal definition retained by the App after registration.
pub struct ModalDefinition<T> {
    pub(crate) id:String,
    pub(crate) title:String,
    pub(crate) inputs:Vec<ModalInput<T>>,
    pub(crate) submit:Execution<T,ModalSubmission>,
}
/// Registered immutable Modal handle.
pub struct RegisteredModal<T> {
    pub(crate) definition:Arc<ModalDefinition<T>>,
}
/// Modal response payload ready to be sent as an interaction response.
pub struct ModalRender {
    pub(crate) custom_id:String,
    pub(crate) title:String,
    pub(crate) components:Vec<serenity::all::CreateActionRow<'static>>,
}

impl<T> Modal<T> {
    /// Starts a Modal definition with a globally unique ID.
    pub fn new(id:impl Into<String>)->Self {
        Self { id:id.into(),title:String::new(),inputs:Vec::new(),submit:None }
    }
    /// Sets the visible Modal title, subject to Discord's title length limit.
    pub fn set_title(mut self,value:impl Into<String>)->Self {
        self.title=value.into(); self
    }
    /// Appends one supported input definition. Modals support at most five top-level inputs.
    pub fn add_input(mut self,value:ModalInput<T>)->Self {
        self.inputs.push(value); self
    }
    /// Appends a Text Input with a stable submitted field ID.
    pub fn add_text_input(
        self,id:impl Into<String>,label:impl IntoValue<String,T>,style:TextInputStyle,
    )->Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::Text(TextInput {
            id: id.into(),
            label: label.into_value(),
            style,
            required: true,
            placeholder: None,
            min_length: None,
            max_length: None,
        }));
        modal
    }
    /// Appends a String Select input using stable option values.
    pub fn add_select(
        self,id:impl Into<String>,label:impl IntoValue<String,T>,
        options:impl IntoIterator<Item=ModalOption<T>>,
    )->Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::StringSelect(ModalSelectInput {
            id: id.into(),
            label: label.into_value(),
            options: options.into_iter().collect(),
        }));
        modal
    }
    /// Appends a single Checkbox input.
    pub fn add_checkbox(self,id:impl Into<String>,label:impl IntoValue<String,T>)->Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::Checkbox(CheckboxInput {
            id: id.into(),
            label: label.into_value(),
            default: false,
        }));
        modal
    }
    /// Appends a Checkbox Group input.
    pub fn add_checkbox_group(
        self,id:impl Into<String>,label:impl IntoValue<String,T>,
        options:impl IntoIterator<Item=ModalOption<T>>,
    )->Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::CheckboxGroup(CheckboxGroupInput {
            id: id.into(),
            label: label.into_value(),
            options: options.into_iter().collect(),
        }));
        modal
    }
    /// Appends a Radio Group input.
    pub fn add_radio_group(
        self,id:impl Into<String>,label:impl IntoValue<String,T>,
        options:impl IntoIterator<Item=ModalOption<T>>,
    )->Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::RadioGroup(RadioGroupInput {
            id: id.into(),
            label: label.into_value(),
            options: options.into_iter().collect(),
        }));
        modal
    }
    /// Appends a File Upload input.
    pub fn add_file_upload(
        self,id:impl Into<String>,label:impl IntoValue<String,T>,
    )->Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::FileUpload(FileUploadInput {
            id: id.into(),
            label: label.into_value(),
            required: true,
        }));
        modal
    }
    /// Sets the raw submission callback. It receives Ctx, the Modal's original
    /// data, and submitted fields separately.
    pub fn on_submit<F,Fut>(mut self,callback:F)->Self
    where
        F:Fn(Ctx,T,ModalSubmission)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static
{
        self.submit = Some(Execution::with_input("submit", callback));
        self
    }
    /// Sets a typed submission callback. Submitted fields are deserialized into
    /// `S` through Serde before the callback runs.
    pub fn on_submit_typed<S,F,Fut>(mut self,callback:F)->Self
    where
        S:DeserializeOwned+Send+'static,
        F:Fn(Ctx,T,S)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static
{
        let callback = Arc::new(callback);
        self.submit = Some(Execution::with_input(
            "submit",
            move |ctx, data, submission: ModalSubmission| {
                let callback = Arc::clone(&callback);
                async move {
                    let form = submission.deserialize::<S>()?;
                    callback(ctx, data, form).await
                }
            },
        ));
        self
    }
    /// Validates and registers this Modal definition. Registration consumes the
    /// builder and rejects duplicate Modal IDs.
    pub fn register(self,app:&App)->Result<RegisteredModal<T>>
    where T:Serialize+DeserializeOwned+Send+Sync+'static
{
        anyhow::ensure!(!self.id.trim().is_empty(), "Modal ID must not be empty");
        anyhow::ensure!(!self.title.trim().is_empty(), "Modal title must not be empty");
        anyhow::ensure!(
            self.title.chars().count() <= crate::page::limits::MODAL_TITLE_MAX_CHARS,
            "Modal title exceeds {} characters",
            crate::page::limits::MODAL_TITLE_MAX_CHARS
        );
        anyhow::ensure!(
            self.inputs.len() <= crate::page::limits::MODAL_MAX_TOP_LEVEL_COMPONENTS,
            "A Modal cannot contain more than {} top-level inputs",
            crate::page::limits::MODAL_MAX_TOP_LEVEL_COMPONENTS
        );
        let mut input_ids = std::collections::HashSet::new();
        for input in &self.inputs {
            let id = match input {
                ModalInput::Text(v) => &v.id,
                ModalInput::StringSelect(v)
                | ModalInput::UserSelect(v)
                | ModalInput::RoleSelect(v)
                | ModalInput::MentionableSelect(v)
                | ModalInput::ChannelSelect(v) => &v.id,
                ModalInput::Checkbox(v) => &v.id,
                ModalInput::CheckboxGroup(v) => &v.id,
                ModalInput::RadioGroup(v) => &v.id,
                ModalInput::FileUpload(v) => &v.id,
            };
            anyhow::ensure!(!id.trim().is_empty(), "Modal input IDs must not be empty");
            anyhow::ensure!(input_ids.insert(id.clone()), "Duplicate Modal input ID: {id}");
        }
        let submit = self.submit.ok_or_else(|| anyhow::anyhow!("Modal requires an on_submit callback"))?;
        let definition = Arc::new(ModalDefinition {
            id: self.id,
            title: self.title,
            inputs: self.inputs,
            submit,
        });
        app.modals.insert(definition.clone())?;
        Ok(RegisteredModal { definition })
    }
}

impl<T> RegisteredModal<T>
where T:Serialize+DeserializeOwned+Send+Sync+'static
{
    /// Serializes the supplied Modal data and prepares the Modal interaction response.
    pub async fn render(&self,app:&App,data:T)->Result<ModalRender> {
        todo!("persist original Modal data and prepare Discord Modal response")
    }
}
