//! Rendered output is the framework/Serenity boundary.
use std::sync::Arc;
use serenity::builder::CreateComponent;
use super::frame::pagination::PaginationState;

pub struct RenderedComponent {
    pub(crate) component:CreateComponent<'static>,
}
pub struct OutgoingAttachment {
    pub filename:String,
    pub bytes:Arc<Vec<u8>>,
}
pub struct RenderedMessage {
    pub(crate) components:Vec<RenderedComponent>,
    pub(crate) attachments:Vec<OutgoingAttachment>,
    pub(crate) pagination:Option<PaginationState>,
}
pub struct RenderedPage {
    pub pages:Vec<RenderedMessage>,
    pub current_index:usize,
}
impl RenderedMessage {
    pub fn components(&self)->&[RenderedComponent] { &self.components }
    pub fn attachments(&self)->&[OutgoingAttachment] { &self.attachments }
    pub fn pagination(&self)->Option<&PaginationState> { self.pagination.as_ref() }
    pub fn add_attachment(mut self,filename:impl Into<String>,bytes:Vec<u8>)->Self {
        self.attachments.push(OutgoingAttachment { filename:filename.into(),bytes:Arc::new(bytes) });
        self
    }
}

pub(crate) fn validate_component<T,E>(
    _component:&super::component::Component<T,E>
)->anyhow::Result<()> {
    todo!("validate resolved component tree and Discord component budgets")
}
