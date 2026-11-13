//! Discord-submitted Modal values, separate from persisted Modal data.
use std::collections::HashMap;
use serde::de::DeserializeOwned;
use crate::page::execution::{SelectedChannel,SelectedMentionable,SelectedRole,SelectedUser};

#[derive(Debug,Clone)]
/// One value returned by a submitted Modal input.
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
/// All submitted Modal input values, indexed by their stable developer-defined IDs.
pub struct ModalSubmission {
    /// Values keyed by the stable IDs assigned to Modal inputs.
    pub values:HashMap<String,SubmittedValue>,
}
impl ModalSubmission {
    /// Reads a Text Input value by its input ID; missing or wrong-type values are errors.
    pub fn text(&self,id:&str)->anyhow::Result<&str> {
        todo!("retrieve a Text Input value or return contextual anyhow error")
    }
    /// Reads String Select, Radio Group, or Checkbox Group selections by input ID.
    pub fn strings(&self,id:&str)->anyhow::Result<&[String]> {
        todo!("retrieve String Select, Radio Group, or Checkbox Group selections")
    }
    /// Reads selected Users by input ID.
    pub fn users(&self,id:&str)->anyhow::Result<&[SelectedUser]> {
        todo!("retrieve typed User Select values")
    }
    /// Reads selected Roles by input ID.
    pub fn roles(&self,id:&str)->anyhow::Result<&[SelectedRole]> {
        todo!("retrieve typed Role Select values")
    }
    /// Reads selected Users or Roles by input ID.
    pub fn mentionables(&self,id:&str)->anyhow::Result<&[SelectedMentionable]> {
        todo!("retrieve typed Mentionable Select values")
    }
    /// Reads selected Channels by input ID.
    pub fn channels(&self,id:&str)->anyhow::Result<&[SelectedChannel]> {
        todo!("retrieve typed Channel Select values")
    }
    /// Reads uploaded attachment references by input ID; this does not download their bytes.
    pub fn files(&self,id:&str)->anyhow::Result<&[serenity::all::Attachment]> {
        todo!("retrieve uploaded attachment references")
    }
    /// Reads a Checkbox value by input ID.
    pub fn boolean(&self,id:&str)->anyhow::Result<bool> {
        todo!("retrieve Checkbox state")
    }
    /// Deserializes all submitted values into a developer-defined Serde type.
    /// Field names are matched to input IDs; missing or invalid values are errors.
    pub fn deserialize<T:DeserializeOwned>(&self)->anyhow::Result<T> {
        todo!("map stable input IDs to a Serde representation and deserialize")
    }
}
