//! Displays an attachment already included in the outgoing message.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct File {
    pub filename:String,
}
impl File {
    pub fn new(filename:impl Into<String>)->Self {
        Self { filename:filename.into() }
    }
}
