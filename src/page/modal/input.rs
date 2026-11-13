//! Modal form inputs. Required Discord Label wrappers stay internal.
use crate::page::value::{IntoValue,Value};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TextInputStyle { Short,Paragraph }

pub struct ModalOption<T,E> {
    pub label:Value<String,T,E>,
    pub value:String,
    pub description:Option<Value<String,T,E>>,
}
impl<T,E> ModalOption<T,E> {
    pub fn new<L:IntoValue<String,T,E>>(label:L,value:impl Into<String>)->Self {
        todo!("create a labelled option with a stable submitted value")
    }
    pub fn set_description<V:IntoValue<String,T,E>>(mut self,value:V)->Self {
        self.description=Some(value.into_value()); self
    }
}

pub struct TextInput<T,E> {
    pub id:String,pub label:Value<String,T,E>,pub style:TextInputStyle,
    pub required:bool,pub placeholder:Option<Value<String,T,E>>,
    pub min_length:Option<u16>,pub max_length:Option<u16>,
}
pub struct ModalSelectInput<T,E> {
    pub id:String,pub label:Value<String,T,E>,pub options:Vec<ModalOption<T,E>>,
}
pub struct CheckboxInput<T,E> { pub id:String,pub label:Value<String,T,E>,pub default:bool }
pub struct CheckboxGroupInput<T,E> { pub id:String,pub label:Value<String,T,E>,pub options:Vec<ModalOption<T,E>> }
pub struct RadioGroupInput<T,E> { pub id:String,pub label:Value<String,T,E>,pub options:Vec<ModalOption<T,E>> }
pub struct FileUploadInput<T,E> { pub id:String,pub label:Value<String,T,E>,pub required:bool }

pub enum ModalInput<T,E> {
    Text(TextInput<T,E>),
    StringSelect(ModalSelectInput<T,E>),
    UserSelect(ModalSelectInput<T,E>),
    RoleSelect(ModalSelectInput<T,E>),
    MentionableSelect(ModalSelectInput<T,E>),
    ChannelSelect(ModalSelectInput<T,E>),
    FileUpload(FileUploadInput<T,E>),
    RadioGroup(RadioGroupInput<T,E>),
    CheckboxGroup(CheckboxGroupInput<T,E>),
    Checkbox(CheckboxInput<T,E>),
}
