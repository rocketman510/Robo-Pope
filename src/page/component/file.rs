//! Displays an attachment already included in the outgoing message.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Component reference to an attachment already included in the outgoing message.
/// The component does not load or upload file bytes itself.
pub struct File {
    /// Complete Discord attachment URI, including the `attachment://` scheme.
    pub attachment_uri: String,
    /// Whether Discord marks the referenced attachment as a spoiler.
    pub spoiler: bool,
}

impl File {
    /// References an outgoing attachment using its complete Discord attachment URI.
    pub fn new(attachment_uri: impl Into<String>) -> Self {
        Self {
            attachment_uri: attachment_uri.into(),
            spoiler: false,
        }
    }

    /// Sets whether Discord should mark this File component as a spoiler.
    pub fn set_spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = spoiler;
        self
    }
}
