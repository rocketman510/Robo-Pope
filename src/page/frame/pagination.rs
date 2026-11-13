//! Pagination state and semantic component-budget planning.
use serde::{Deserialize,Serialize};
use crate::page::frame::element::Element;

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
pub struct PaginationState {
    pub frame_id:String,
    pub page_index:usize,
    pub page_count:usize,
}
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]
pub struct ComponentBudget {
    pub total_components:usize,
    pub text_display_chars:usize,
    pub direct_container_children:usize,
}
pub struct PaginationPlan<T,E> {
    pub pages:Vec<Vec<Element<T,E>>>,
    pub state:PaginationState,
}

pub fn plan<T,E>(
    frame_id:impl Into<String>,
    elements:Vec<Element<T,E>>,
    reserved_static:ComponentBudget,
    reserved_navigation:ComponentBudget,
)->anyhow::Result<PaginationPlan<T,E>> {
    todo!("maximize complete elements per page without exceeding nested and semantic limits")
}
