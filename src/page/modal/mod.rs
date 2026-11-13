//! Modal form blueprint, independent of message Pages.
pub mod input;
pub mod submission;

pub use input::{
    CheckboxGroupInput, CheckboxInput, FileUploadInput, ModalInput, ModalOption,
    ModalSelectInput, RadioGroupInput, TextInput, TextInputStyle,
};
pub use submission::{ModalSubmission, SubmittedValue};

use std::future::Future;
use std::sync::Arc;
use anyhow::Context as _;
use serenity::builder::{
    CreateCheckbox, CreateCheckboxGroup, CreateCheckboxGroupOption, CreateFileUpload,
    CreateInputText, CreateLabel, CreateModalComponent, CreateRadioGroup,
    CreateRadioGroupOption, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};
use serenity::all::InputTextStyle as SerenityInputTextStyle;
use crate::app::store::{derive_custom_id, DefinitionKind, StoreRecord};
use serde::{Serialize, de::DeserializeOwned};
use crate::app::{App, Ctx, Result};
use crate::page::execution::Execution;
use crate::page::value::{IntoValue, Value};

/// Mutable builder for a Discord Modal form.
pub struct Modal<T> {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) inputs: Vec<ModalInput<T>>,
    pub(crate) submit: Option<Execution<T, ModalSubmission>>,
}
/// Immutable Modal definition retained by the App after registration.
pub struct ModalDefinition<T> {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) inputs: Vec<ModalInput<T>>,
    pub(crate) submit: Execution<T, ModalSubmission>,
}
/// Registered immutable Modal handle.
pub struct RegisteredModal<T> {
    pub(crate) definition: Arc<ModalDefinition<T>>,
}
/// Modal response payload ready to be sent as an interaction response.
pub struct ModalRender {
    pub(crate) custom_id: String,
    pub(crate) title: String,
    pub(crate) components: Vec<CreateModalComponent<'static>>,
}

fn validate_label(value: &str, definition_id: &str, input_id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !value.trim().is_empty(),
        "Modal '{}' input '{}' label must not be empty",
        definition_id,
        input_id
    );
    anyhow::ensure!(
        value.chars().count() <= crate::page::limits::MODAL_LABEL_MAX_CHARS,
        "Modal '{}' input '{}' label exceeds {} characters",
        definition_id,
        input_id,
        crate::page::limits::MODAL_LABEL_MAX_CHARS
    );
    Ok(())
}

fn validate_description(value: &str, definition_id: &str, input_id: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        value.chars().count() <= crate::page::limits::MODAL_LABEL_DESCRIPTION_MAX_CHARS,
        "Modal '{}' input '{}' option description exceeds {} characters",
        definition_id,
        input_id,
        crate::page::limits::MODAL_LABEL_DESCRIPTION_MAX_CHARS
    );
    Ok(())
}

fn validate_static_options<T>(
    options: &[ModalOption<T>],
    definition_id: &str,
    input_id: &str,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        options.len() <= crate::page::limits::MODAL_SELECT_MAX_OPTIONS,
        "Modal '{}' input '{}' has {} options; maximum is {}",
        definition_id,
        input_id,
        options.len(),
        crate::page::limits::MODAL_SELECT_MAX_OPTIONS
    );
    let mut values = std::collections::HashSet::new();
    for option in options {
        anyhow::ensure!(
            !option.value.is_empty(),
            "Modal '{}' input '{}' contains an empty option value",
            definition_id,
            input_id
        );
        anyhow::ensure!(
            values.insert(option.value.as_str()),
            "Modal '{}' input '{}' contains duplicate option value '{}'",
            definition_id,
            input_id,
            option.value
        );
        if let Value::Static(label) = &option.label {
            validate_label(label, definition_id, input_id)?;
        }
        if let Some(Value::Static(description)) = &option.description {
            validate_description(description, definition_id, input_id)?;
        }
    }
    Ok(())
}

fn static_label<T>(label: &Value<String, T>, definition_id: &str, input_id: &str) -> anyhow::Result<()> {
    if let Value::Static(label) = label {
        validate_label(label, definition_id, input_id)?;
    }
    Ok(())
}

impl<T> Modal<T> {
    /// Starts a Modal definition with a globally unique ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), title: String::new(), inputs: Vec::new(), submit: None }
    }

    /// Sets the visible Modal title, subject to Discord's title length limit.
    pub fn set_title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }

    /// Appends one supported input definition.
    pub fn add_input(mut self, value: ModalInput<T>) -> Self {
        self.inputs.push(value);
        self
    }

    /// Appends a Text Input with a stable submitted field ID.
    pub fn add_text_input(
        self,
        id: impl Into<String>,
        label: impl IntoValue<String, T>,
        style: TextInputStyle,
    ) -> Self {
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
        self,
        id: impl Into<String>,
        label: impl IntoValue<String, T>,
        options: impl IntoIterator<Item = ModalOption<T>>,
    ) -> Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::StringSelect(ModalSelectInput {
            id: id.into(),
            label: label.into_value(),
            options: options.into_iter().collect(),
        }));
        modal
    }

    /// Appends a single Checkbox input.
    pub fn add_checkbox(self, id: impl Into<String>, label: impl IntoValue<String, T>) -> Self {
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
        self,
        id: impl Into<String>,
        label: impl IntoValue<String, T>,
        options: impl IntoIterator<Item = ModalOption<T>>,
    ) -> Self {
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
        self,
        id: impl Into<String>,
        label: impl IntoValue<String, T>,
        options: impl IntoIterator<Item = ModalOption<T>>,
    ) -> Self {
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
        self,
        id: impl Into<String>,
        label: impl IntoValue<String, T>,
    ) -> Self {
        let mut modal = self;
        modal.inputs.push(ModalInput::FileUpload(FileUploadInput {
            id: id.into(),
            label: label.into_value(),
            required: true,
        }));
        modal
    }

    /// Sets the raw submission callback.
    pub fn on_submit<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(Ctx, T, ModalSubmission) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static,
    {
        self.submit = Some(Execution::with_input("submit", callback));
        self
    }

    /// Sets a typed submission callback, deserializing submitted fields through Serde.
    pub fn on_submit_typed<S, F, Fut>(mut self, callback: F) -> Self
    where
        S: DeserializeOwned + Send + 'static,
        F: Fn(Ctx, T, S) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
        T: Send + 'static,
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

    /// Validates and registers this Modal definition.
    pub fn register(self, app: &App) -> Result<RegisteredModal<T>>
    where
        T: Serialize + DeserializeOwned + Send + Sync + 'static,
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
            let (id, label) = match input {
                ModalInput::Text(v) => {
                    if let Some(max) = v.max_length {
                        anyhow::ensure!(
                            max as usize <= crate::page::limits::MODAL_TEXT_INPUT_MAX_CHARS,
                            "Modal '{}' Text Input '{}' maximum length exceeds {}",
                            self.id, v.id, crate::page::limits::MODAL_TEXT_INPUT_MAX_CHARS
                        );
                    }
                    if let (Some(min), Some(max)) = (v.min_length, v.max_length) {
                        anyhow::ensure!(
                            min <= max,
                            "Modal '{}' Text Input '{}' minimum length exceeds maximum",
                            self.id, v.id
                        );
                    }
                    (&v.id, &v.label)
                }
                ModalInput::StringSelect(v)
                | ModalInput::UserSelect(v)
                | ModalInput::RoleSelect(v)
                | ModalInput::MentionableSelect(v)
                | ModalInput::ChannelSelect(v) => {
                    validate_static_options(&v.options, &self.id, &v.id)?;
                    (&v.id, &v.label)
                }
                ModalInput::Checkbox(v) => (&v.id, &v.label),
                ModalInput::CheckboxGroup(v) => {
                    validate_static_options(&v.options, &self.id, &v.id)?;
                    (&v.id, &v.label)
                }
                ModalInput::RadioGroup(v) => {
                    validate_static_options(&v.options, &self.id, &v.id)?;
                    (&v.id, &v.label)
                }
                ModalInput::FileUpload(v) => (&v.id, &v.label),
            };
            anyhow::ensure!(!id.trim().is_empty(), "Modal input IDs must not be empty");
            anyhow::ensure!(input_ids.insert(id.clone()), "Duplicate Modal input ID: {id}");
            static_label(label, &self.id, id)?;
        }

        let submit = self.submit.ok_or_else(|| anyhow::anyhow!("Modal requires an on_submit callback"))?;
        app.reserve_definition_id(&self.id)?;
        if let Err(error) = submit.register(app, &self.id, None) {
            app.executions.remove_definition(&self.id);
            app.release_definition_id(&self.id);
            return Err(error);
        }
        let definition = Arc::new(ModalDefinition {
            id: self.id,
            title: self.title,
            inputs: self.inputs,
            submit,
        });
        if let Err(error) = app.modals.insert(definition.clone()) {
            app.executions.remove_definition(&definition.id);
            app.release_definition_id(&definition.id);
            return Err(error);
        }
        Ok(RegisteredModal { definition })
    }
}

impl<T> RegisteredModal<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    /// Resolves and validates every Modal input, then persists its submit record.
    ///
    /// Store insertion is intentionally the final step. A failed dynamic value or
    /// invalid resolved component cannot leave an orphaned custom-ID record.
    pub async fn render(&self, app: &App, data: T) -> Result<ModalRender> {
        use self::input::ModalInput as I;

        let serialized_data = serde_json::to_vec(&data)
            .with_context(|| format!("Serializing data for Modal '{}'", self.definition.id))?;
        let custom_id = derive_custom_id(
            &self.definition.id,
            &self.definition.submit.id,
            &serialized_data,
            None,
        ).with_context(|| format!("Deriving custom ID for Modal '{}'", self.definition.id))?;

        let ctx = app.render_context(self.definition.id.clone());
        let data = Arc::new(data);
        let mut components = Vec::with_capacity(self.definition.inputs.len());

        for (input_index, input) in self.definition.inputs.iter().enumerate() {
            let (id, label_value) = match input {
                I::Text(v) => (&v.id, &v.label),
                I::StringSelect(v) | I::UserSelect(v) | I::RoleSelect(v)
                | I::MentionableSelect(v) | I::ChannelSelect(v) => (&v.id, &v.label),
                I::Checkbox(v) => (&v.id, &v.label),
                I::CheckboxGroup(v) => (&v.id, &v.label),
                I::RadioGroup(v) => (&v.id, &v.label),
                I::FileUpload(v) => (&v.id, &v.label),
            };
            let label = super::value::resolve(label_value, ctx.clone(), data.clone())
                .await.with_context(|| format!(
                    "Resolving label for Modal '{}' input '{}'", self.definition.id, id
                ))?;
            validate_label(&label, &self.definition.id, id)?;

            let rendered = match input {
                I::Text(input) => {
                    let style = match input.style {
                        TextInputStyle::Short => SerenityInputTextStyle::Short,
                        TextInputStyle::Paragraph => SerenityInputTextStyle::Paragraph,
                    };
                    let mut builder = CreateInputText::new(style, input.id.clone())
                        .required(input.required);
                    if let Some(placeholder) = &input.placeholder {
                        let placeholder = super::value::resolve(
                            placeholder, ctx.clone(), data.clone()
                        ).await.with_context(|| format!(
                            "Resolving placeholder for Modal '{}' input '{}'",
                            self.definition.id, input.id
                        ))?;
                        anyhow::ensure!(
                            placeholder.chars().count() <= crate::page::limits::MODAL_LABEL_MAX_CHARS,
                            "Modal '{}' Text Input '{}' placeholder exceeds {} characters",
                            self.definition.id, input.id, crate::page::limits::MODAL_LABEL_MAX_CHARS
                        );
                        builder = builder.placeholder(placeholder);
                    }
                    if let Some(min) = input.min_length { builder = builder.min_length(min); }
                    if let Some(max) = input.max_length { builder = builder.max_length(max); }
                    CreateModalComponent::Label(CreateLabel::input_text(label, builder).into_owned())
                }
                I::StringSelect(input) | I::UserSelect(input) | I::RoleSelect(input)
                | I::MentionableSelect(input) | I::ChannelSelect(input) => {
                    let mut options = Vec::with_capacity(input.options.len());
                    for option in &input.options {
                        let option_label = super::value::resolve(&option.label, ctx.clone(), data.clone())
                            .await.with_context(|| format!(
                                "Resolving Modal '{}' input '{}' option label", self.definition.id, input.id
                            ))?;
                        validate_label(&option_label, &self.definition.id, &input.id)?;
                        let mut rendered_option = CreateSelectMenuOption::new(option_label, option.value.clone());
                        if let Some(description) = &option.description {
                            let description = super::value::resolve(
                                description, ctx.clone(), data.clone()
                            ).await.with_context(|| format!(
                                "Resolving Modal '{}' input '{}' option description", self.definition.id, input.id
                            ))?;
                            validate_description(&description, &self.definition.id, &input.id)?;
                            rendered_option = rendered_option.description(description);
                        }
                        options.push(rendered_option);
                    }
                    let kind = match &self.definition.inputs[input_index] {
                        I::StringSelect(_) => CreateSelectMenuKind::String { options: options.into() },
                        I::UserSelect(_) => CreateSelectMenuKind::User { default_users: None },
                        I::RoleSelect(_) => CreateSelectMenuKind::Role { default_roles: None },
                        I::MentionableSelect(_) => CreateSelectMenuKind::Mentionable {
                            default_users: None, default_roles: None,
                        },
                        I::ChannelSelect(_) => CreateSelectMenuKind::Channel {
                            channel_types: None, default_channels: None,
                        },
                        _ => unreachable!("this match arm only handles select inputs"),
                    };
                    CreateModalComponent::Label(
                        CreateLabel::select_menu(label, CreateSelectMenu::new(input.id.clone(), kind).required(true))
                            .into_owned()
                    )
                }
                I::FileUpload(input) => CreateModalComponent::Label(
                    CreateLabel::file_upload(
                        label,
                        CreateFileUpload::new(input.id.clone()).required(input.required),
                    ).into_owned()
                ),
                I::RadioGroup(input) => {
                    let mut options = Vec::with_capacity(input.options.len());
                    for option in &input.options {
                        let option_label = super::value::resolve(&option.label, ctx.clone(), data.clone()).await?;
                        validate_label(&option_label, &self.definition.id, &input.id)?;
                        let mut rendered = CreateRadioGroupOption::new(option_label, option.value.clone());
                        if let Some(description) = &option.description {
                            let description = super::value::resolve(description, ctx.clone(), data.clone()).await?;
                            validate_description(&description, &self.definition.id, &input.id)?;
                            rendered = rendered.description(description);
                        }
                        options.push(rendered);
                    }
                    CreateModalComponent::Label(CreateLabel::radio_group(
                        label, CreateRadioGroup::new(input.id.clone(), options).required(true),
                    ).into_owned())
                }
                I::CheckboxGroup(input) => {
                    let mut options = Vec::with_capacity(input.options.len());
                    for option in &input.options {
                        let option_label = super::value::resolve(&option.label, ctx.clone(), data.clone()).await?;
                        validate_label(&option_label, &self.definition.id, &input.id)?;
                        let mut rendered = CreateCheckboxGroupOption::new(option_label, option.value.clone());
                        if let Some(description) = &option.description {
                            let description = super::value::resolve(description, ctx.clone(), data.clone()).await?;
                            validate_description(&description, &self.definition.id, &input.id)?;
                            rendered = rendered.description(description);
                        }
                        options.push(rendered);
                    }
                    CreateModalComponent::Label(CreateLabel::checkbox_group(
                        label, CreateCheckboxGroup::new(input.id.clone(), options),
                    ).into_owned())
                }
                I::Checkbox(input) => CreateModalComponent::Label(CreateLabel::checkbox(
                    label, CreateCheckbox::new(input.id.clone()).default_selected(input.default),
                ).into_owned()),
            };
            components.push(rendered);
        }

        let record = StoreRecord {
            custom_id: custom_id.0.clone(),
            definition_kind: DefinitionKind::Modal,
            definition_id: self.definition.id.clone(),
            execution_id: self.definition.submit.id.clone(),
            serialized_data: serialized_data.clone(),
            frame_identity: None,
            pagination_state: None,
            schema_version: 1,
        };
        app.store().insert(record).await.with_context(|| format!(
            "Persisting submit execution for Modal '{}'", self.definition.id
        ))?;

        Ok(ModalRender {
            custom_id: custom_id.0,
            title: self.definition.title.clone(),
            components,
        })
    }
}
