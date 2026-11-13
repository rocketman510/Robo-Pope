//! Pagination state and semantic component-budget planning.
use serde::{Deserialize,Serialize};
use crate::page::frame::element::Element;

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq)]
/// Persistable pagination position for a specific Frame.
pub struct PaginationState {
    pub frame_id:String,
    pub page_index:usize,
    pub page_count:usize,
}
#[derive(Debug,Clone,Copy,Default,PartialEq,Eq)]
/// Portion of a Discord message budget reserved by static content or navigation.
/// Counts remain separate because component, Text Display, and Container limits differ.
pub struct ComponentBudget {
    pub total_components:usize,
    pub text_display_chars:usize,
    pub direct_container_children:usize,
}
/// Complete pagination result, preserving each element intact and its page state.
pub struct PaginationPlan<T> {
    pub pages:Vec<Vec<Element<T>>>,
    pub state:PaginationState,
}

/// Splits Frame elements into the largest valid pages after subtracting reserved
/// static and navigation budgets. It never truncates or splits an element.
pub fn plan<T>(
    frame_id:impl Into<String>,
    elements:Vec<Element<T>>,
    reserved_static:ComponentBudget,
    reserved_navigation:ComponentBudget,
)->anyhow::Result<PaginationPlan<T>> {
    todo!("maximize complete elements per page without exceeding nested and semantic limits")
}
