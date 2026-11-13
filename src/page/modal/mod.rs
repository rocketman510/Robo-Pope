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

pub struct Modal<T,E=()> {
    pub(crate) id:String,
    pub(crate) title:String,
    pub(crate) inputs:Vec<ModalInput<T,E>>,
    pub(crate) submit:Option<Execution<T,E,ModalSubmission>>,
}
pub struct ModalDefinition<T,E> {
    pub(crate) id:String,
    pub(crate) title:String,
    pub(crate) inputs:Vec<ModalInput<T,E>>,
    pub(crate) submit:Execution<T,E,ModalSubmission>,
}
pub struct RegisteredModal<T,E=()> {
    pub(crate) definition:Arc<ModalDefinition<T,E>>,
}
pub struct ModalRender {
    pub(crate) custom_id:String,
    pub(crate) title:String,
    pub(crate) components:Vec<serenity::all::CreateActionRow>,
}

impl<T,E> Modal<T,E> {
    pub fn new(id:impl Into<String>)->Self {
        Self { id:id.into(),title:String::new(),inputs:Vec::new(),submit:None }
    }
    pub fn set_title(mut self,value:impl Into<String>)->Self {
        self.title=value.into(); self
    }
    pub fn add_input(mut self,value:ModalInput<T,E>)->Self {
        self.inputs.push(value); self
    }
    pub fn add_text_input(
        self,id:impl Into<String>,label:impl IntoValue<String,T,E>,style:TextInputStyle,
    )->Self {
        todo!("append a Text Input definition")
    }
    pub fn add_select(
        self,id:impl Into<String>,label:impl IntoValue<String,T,E>,
        options:impl IntoIterator<Item=ModalOption<T,E>>,
    )->Self {
        todo!("append a String Select input")
    }
    pub fn add_checkbox(self,id:impl Into<String>,label:impl IntoValue<String,T,E>)->Self {
        todo!("append a Checkbox input")
    }
    pub fn add_checkbox_group(
        self,id:impl Into<String>,label:impl IntoValue<String,T,E>,
        options:impl IntoIterator<Item=ModalOption<T,E>>,
    )->Self {
        todo!("append Checkbox Group input")
    }
    pub fn add_radio_group(
        self,id:impl Into<String>,label:impl IntoValue<String,T,E>,
        options:impl IntoIterator<Item=ModalOption<T,E>>,
    )->Self {
        todo!("append Radio Group input")
    }
    pub fn add_file_upload(
        self,id:impl Into<String>,label:impl IntoValue<String,T,E>,
    )->Self {
        todo!("append File Upload input")
    }
    pub fn on_submit<F,Fut>(mut self,callback:F)->Self
    where
        F:Fn(Ctx<E>,T,ModalSubmission)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static,E:Send+Sync+'static,
    {
        todo!("attach raw ModalSubmission callback")
    }
    pub fn on_submit_typed<S,F,Fut>(mut self,callback:F)->Self
    where
        S:DeserializeOwned+Send+'static,
        F:Fn(Ctx<E>,T,S)->Fut+Send+Sync+'static,
        Fut:Future<Output=Result<()>>+Send+'static,
        T:Send+'static,E:Send+Sync+'static,
    {
        todo!("deserialize typed form from ModalSubmission before invoking callback")
    }
    pub fn register(self,app:&App<E>)->Result<RegisteredModal<T,E>>
    where T:Serialize+DeserializeOwned+Send+Sync+'static,E:Send+Sync+'static
    {
        todo!("consume builder, validate inputs, register immutable definition")
    }
}

impl<T,E> RegisteredModal<T,E>
where T:Serialize+DeserializeOwned+Send+Sync+'static,E:Send+Sync+'static
{
    pub async fn render(&self,app:&App<E>,data:T)->Result<ModalRender> {
        todo!("persist original Modal data and prepare Discord Modal response")
    }
}
