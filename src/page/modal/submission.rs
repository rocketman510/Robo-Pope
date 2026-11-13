//! Discord-submitted Modal values, separate from persisted Modal data.
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use crate::page::execution::{SelectedChannel,SelectedMentionable,SelectedRole,SelectedUser};

#[derive(Debug,Clone)]
pub enum SubmittedValue {
    Text(String),
    Strings(Vec<String>),
    Users(Vec<SelectedUser>),
    Roles(Vec<SelectedRole>),
    Mentionables(Vec<SelectedMentionable>),
    Channels(Vec<SelectedChannel>),
    Files(Vec<serenity::all::Attachment>),
    Boolean(bool),
}
#[derive(Debug,Clone,Default)]
pub struct ModalSubmission {
    pub values:HashMap<String,SubmittedValue>,
}
impl ModalSubmission {
    pub fn text(&self,id:&str)->anyhow::Result<&str> {
        todo!("retrieve a Text Input value or return contextual anyhow error")
    }
    pub fn strings(&self,id:&str)->anyhow::Result<&[String]> {
        todo!("retrieve String Select, Radio Group, or Checkbox Group selections")
    }
    pub fn users(&self,id:&str)->anyhow::Result<&[SelectedUser]> {
        todo!("retrieve typed User Select values")
    }
    pub fn roles(&self,id:&str)->anyhow::Result<&[SelectedRole]> {
        todo!("retrieve typed Role Select values")
    }
    pub fn mentionables(&self,id:&str)->anyhow::Result<&[SelectedMentionable]> {
        todo!("retrieve typed Mentionable Select values")
    }
    pub fn channels(&self,id:&str)->anyhow::Result<&[SelectedChannel]> {
        todo!("retrieve typed Channel Select values")
    }
    pub fn files(&self,id:&str)->anyhow::Result<&[serenity::all::Attachment]> {
        todo!("retrieve uploaded attachment references")
    }
    pub fn boolean(&self,id:&str)->anyhow::Result<bool> {
        todo!("retrieve Checkbox state")
    }
    pub fn deserialize<T:DeserializeOwned>(&self)->anyhow::Result<T> {
        todo!("map stable input IDs to a Serde representation and deserialize")
    }
}
