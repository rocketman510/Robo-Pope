//! Discord-submitted Modal values, separate from persisted Modal data.
use std::collections::HashMap;
use anyhow::Context;
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
        match self.values.get(id) {
            Some(SubmittedValue::Text(value)) => Ok(value.as_str()),
            Some(_) => anyhow::bail!("Modal field '{id}' was submitted with a value that is not text"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads String Select, Radio Group, or Checkbox Group selections by input ID.
    pub fn strings(&self,id:&str)->anyhow::Result<&[String]> {
        match self.values.get(id) {
            Some(SubmittedValue::Strings(values)) => Ok(values.as_slice()),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as a string selection"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads selected Users by input ID.
    pub fn users(&self,id:&str)->anyhow::Result<&[SelectedUser]> {
        match self.values.get(id) {
            Some(SubmittedValue::Users(values)) => Ok(values.as_slice()),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as a User selection"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads selected Roles by input ID.
    pub fn roles(&self,id:&str)->anyhow::Result<&[SelectedRole]> {
        match self.values.get(id) {
            Some(SubmittedValue::Roles(values)) => Ok(values.as_slice()),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as a Role selection"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads selected Users or Roles by input ID.
    pub fn mentionables(&self,id:&str)->anyhow::Result<&[SelectedMentionable]> {
        match self.values.get(id) {
            Some(SubmittedValue::Mentionables(values)) => Ok(values.as_slice()),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as a Mentionable selection"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads selected Channels by input ID.
    pub fn channels(&self,id:&str)->anyhow::Result<&[SelectedChannel]> {
        match self.values.get(id) {
            Some(SubmittedValue::Channels(values)) => Ok(values.as_slice()),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as a Channel selection"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads uploaded attachment references by input ID; this does not download their bytes.
    pub fn files(&self,id:&str)->anyhow::Result<&[serenity::all::Attachment]> {
        match self.values.get(id) {
            Some(SubmittedValue::Files(values)) => Ok(values.as_slice()),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as file uploads"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Reads a Checkbox value by input ID.
    pub fn boolean(&self,id:&str)->anyhow::Result<bool> {
        match self.values.get(id) {
            Some(SubmittedValue::Boolean(value)) => Ok(*value),
            Some(_) => anyhow::bail!("Modal field '{id}' was not submitted as a Boolean"),
            None => anyhow::bail!("Modal submission is missing field '{id}'"),
        }
    }
    /// Deserializes all submitted values into a developer-defined Serde type.
    /// Field names are matched to input IDs; missing or invalid values are errors.
    pub fn deserialize<T:DeserializeOwned>(&self)->anyhow::Result<T> {
        let mut object = serde_json::Map::new();
        for (id, value) in &self.values {
            let json = match value {
                SubmittedValue::Text(value) => serde_json::Value::String(value.clone()),
                SubmittedValue::Strings(values) => serde_json::to_value(values)
                    .context("Serializing Modal string selections")?,
                SubmittedValue::Boolean(value) => serde_json::Value::Bool(*value),
                SubmittedValue::Users(_)
                | SubmittedValue::Roles(_)
                | SubmittedValue::Mentionables(_)
                | SubmittedValue::Channels(_)
                | SubmittedValue::Files(_) => {
                    anyhow::bail!("Modal field '{id}' uses a typed value that is not supported by generic Serde deserialization")
                }
            };
            object.insert(id.clone(), json);
        }
        serde_json::from_value(serde_json::Value::Object(object))
            .map_err(|error| anyhow::anyhow!("Deserializing Modal submission by input ID: {error}"))
    }
}
