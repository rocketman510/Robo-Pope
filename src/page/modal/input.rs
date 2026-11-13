//! Modal form inputs. Required Discord Label wrappers stay internal.
use crate::page::value::{IntoValue,Value};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TextInputStyle { Short,Paragraph }

pub struct ModalOption<T> {
    pub label:Value<String,T>,
    pub value:String,
    pub description:Option<Value<String,T>>,
}
impl<T> ModalOption<T> {
    pub fn new<L:IntoValue<String,T>>(label:L,value:impl Into<String>)->Self {
        todo!("create a labelled option with a stable submitted value")
    }
    pub fn set_description<V:IntoValue<String,T>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
}

pub struct TextInput<T> {
    pub id:String,pub label:Value<String,T>,pub style:TextInputStyle,
    pub required:bool,pub placeholder:Option<Value<String,T>>,
    pub min_length:Option<u16>,pub max_length:Option<u16>,
}
pub struct ModalSelectInput<T> {
    pub id:String,pub label:Value<String,T>,pub options:Vec<ModalOption<T>>,
}
pub struct CheckboxInput<T> { pub id:String,pub label:Value<String,T>,pub default:bool }
pub struct CheckboxGroupInput<T> { pub id:String,pub label:Value<String,T>,pub options:Vec<ModalOption<T>> }
pub struct RadioGroupInput<T> { pub id:String,pub label:Value<String,T>,pub options:Vec<ModalOption<T>> }
pub struct FileUploadInput<T> { pub id:String,pub label:Value<String,T>,pub required:bool }

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
