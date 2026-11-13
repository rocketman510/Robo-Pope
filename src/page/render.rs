//! Rendered output is the framework/Serenity boundary.
use std::sync::Arc;
use serenity::builder::CreateComponent;
use super::frame::pagination::PaginationState;

/// One Serenity component produced by the render layer.
pub struct RenderedComponent {
    pub(crate) component:CreateComponent<'static>,
}
/// File bytes to attach to the outgoing Discord message.
/// File components refer to attachments by filename; they do not contain the bytes.
pub struct OutgoingAttachment {
    pub filename:String,
    pub bytes:Arc<Vec<u8>>,
}
/// Complete Components V2 message payload plus its attachments and pagination metadata.
pub struct RenderedMessage {
    pub(crate) components:Vec<RenderedComponent>,
    pub(crate) attachments:Vec<OutgoingAttachment>,
    pub(crate) pagination:Option<PaginationState>,
}
/// All rendered message pages for one Page instance and its current position.
pub struct RenderedPage {
    pub pages:Vec<RenderedMessage>,
    pub current_index:usize,
}
impl RenderedMessage {
    /// Borrows the rendered component tree.
    pub fn components(&self)->&[RenderedComponent] { &self.components }
    /// Borrows outgoing attachment bytes and filenames.
    pub fn attachments(&self)->&[OutgoingAttachment] { &self.attachments }
    /// Returns pagination metadata when this message represents a paginated Frame.
    pub fn pagination(&self)->Option<&PaginationState> { self.pagination.as_ref() }
    /// Adds outgoing file bytes. A File component can reference the same filename.
    pub fn add_attachment(mut self,filename:impl Into<String>,bytes:Vec<u8>)->Self {
        self.attachments.push(OutgoingAttachment { filename:filename.into(),bytes:Arc::new(bytes) });
        self
    }
}

/// Validates a resolved component tree against the Discord component and text budgets.
/// Validation occurs after dynamic properties and Frame contents have been expanded.
pub(crate) fn validate_component<T>(
    _component:&super::component::Component<T>
)->anyhow::Result<()> {
    todo!("validate resolved component tree and Discord component budgets")
}
