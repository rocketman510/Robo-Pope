//! Modal form inputs. Required Discord Label wrappers stay internal.
use crate::page::value::{IntoValue,Value};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
/// Discord Text Input style.
pub enum TextInputStyle { Short,Paragraph }

/// One Modal select/group option, with a display label and submitted value.
pub struct ModalOption<T> {
    pub label:Value<String,T>,
    pub value:String,
    pub description:Option<Value<String,T>>,
}
impl<T> ModalOption<T> {
    /// Creates an option whose submitted value is independent of its display label.
    pub fn new<L:IntoValue<String,T>>(label:L,value:impl Into<String>)->Self {
        Self {
            label: label.into_value(),
            value: value.into(),
            description: None,
        }
    }
    /// Sets an optional static or dynamic description.
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
}

/// Text Input definition with a stable field ID and Discord length constraints.
pub struct TextInput<T> {
    pub id:String,pub label:Value<String,T>,pub style:TextInputStyle,
    pub required:bool,pub placeholder:Option<Value<String,T>>,
    pub min_length:Option<u16>,pub max_length:Option<u16>,
}
/// Select-style Modal input and its available options.
pub struct ModalSelectInput<T> {
    pub id:String,pub label:Value<String,T>,pub options:Vec<ModalOption<T>>,
}
/// Single Checkbox input definition.
pub struct CheckboxInput<T> { pub id:String,pub label:Value<String,T>,pub default:bool }
/// Checkbox Group input definition.
pub struct CheckboxGroupInput<T> { pub id:String,pub label:Value<String,T>,pub options:Vec<ModalOption<T>> }
/// Radio Group input definition.
pub struct RadioGroupInput<T> { pub id:String,pub label:Value<String,T>,pub options:Vec<ModalOption<T>> }
/// File Upload input definition. Submitted values are attachment references.
pub struct FileUploadInput<T> { pub id:String,pub label:Value<String,T>,pub required:bool }

/// Supported Modal input definitions. Discord Label wrappers are added by rendering.
pub enum ModalInput<T> {
    Text(TextInput<T>),
    StringSelect(ModalSelectInput<T>),
    UserSelect(ModalSelectInput<T>),
    RoleSelect(ModalSelectInput<T>),
    MentionableSelect(ModalSelectInput<T>),
    ChannelSelect(ModalSelectInput<T>),
    FileUpload(FileUploadInput<T>),
    RadioGroup(RadioGroupInput<T>),
    CheckboxGroup(CheckboxGroupInput<T>),
    Checkbox(CheckboxInput<T>),
}
