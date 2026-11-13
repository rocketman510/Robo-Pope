//! Displays an attachment already included in the outgoing message.
#[derive(Debug,Clone,PartialEq,Eq)]
/// Component reference to an attachment already included in the outgoing message.
/// The component does not load or upload file bytes itself.
pub struct File {
    /// Attachment filename referenced by the rendered File component.
    pub filename:String,
}
impl File {
    /// References the outgoing attachment with this filename.
    pub fn new(filename:impl Into<String>)->Self {
        Self { filename:filename.into() }
    }
}
