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
pub struct PaginationPlan<T, D = std::sync::Arc<()>> {
    pub pages:Vec<Vec<Element<T, D>>>,
    pub state:PaginationState,
}

/// Splits Frame elements into the largest valid pages after subtracting reserved
/// static and navigation budgets. It never truncates or splits an element.
pub fn plan<T, D>(
    frame_id:impl Into<String>,
    elements:Vec<Element<T, D>>,
    reserved_static:ComponentBudget,
    reserved_navigation:ComponentBudget,
)->anyhow::Result<PaginationPlan<T, D>> {
    use crate::page::component::{Component, ContainerItem};
    use crate::page::value::{Slot, Value};

    fn add(left: ComponentBudget, right: ComponentBudget) -> ComponentBudget {
        ComponentBudget {
            total_components: left.total_components.saturating_add(right.total_components),
            text_display_chars: left.text_display_chars.saturating_add(right.text_display_chars),
            direct_container_children: left.direct_container_children.max(right.direct_container_children),
        }
    }

    fn fits(budget: ComponentBudget) -> bool {
        budget.total_components <= crate::page::limits::MESSAGE_MAX_COMPONENTS
            && budget.text_display_chars <= crate::page::limits::MESSAGE_MAX_TEXT_DISPLAY_CHARS
            && budget.direct_container_children <= crate::page::limits::CONTAINER_MAX_DIRECT_CHILDREN
    }

    fn measure<T, D>(component: &Component<T, D>) -> anyhow::Result<ComponentBudget> {
        let mut budget = ComponentBudget {
            total_components: 1,
            text_display_chars: 0,
            direct_container_children: 0,
        };
        match component {
            Component::Text(text) => {
                if let Value::Static(value) = &text.content {
                    budget.text_display_chars = value.chars().count();
                }
            }
            Component::Section(section) => {
                let slots = [&section.text.0, &section.text.1, &section.text.2];
                let populated = slots.iter().filter(|slot| matches!(slot, Slot::Value(_))).count();
                anyhow::ensure!(populated > 0, "Section must contain at least one Text Display");
                for slot in slots {
                    if let Slot::Value(Value::Static(value)) = slot {
                        budget.text_display_chars += value.chars().count();
                    }
                }
                if let Some(accessory) = &section.accessory {
                    match accessory {
                        crate::page::SectionAccessory::Thumbnail(_) => budget.total_components += 1,
                        crate::page::SectionAccessory::Button(_) => budget.total_components += 1,
                    }
                }
            }
            Component::Container(container) => {
                anyhow::ensure!(
                    container.children.len() <= crate::page::limits::CONTAINER_MAX_DIRECT_CHILDREN,
                    "Container exceeds its direct-child limit"
                );
                budget.direct_container_children = container.children.len();
                for child in &container.children {
                    match child {
                        ContainerItem::Component(child) => budget = add(budget, measure(child)?),
                        ContainerItem::Frame(_) => anyhow::bail!("Nested Frame contents must be resolved before pagination"),
                    }
                }
            }
            Component::ButtonRow(row) => {
                anyhow::ensure!(
                    (crate::page::limits::BUTTON_ROW_MIN_BUTTONS..=crate::page::limits::BUTTON_ROW_MAX_BUTTONS)
                        .contains(&row.populated()),
                    "Button Action Row must contain between one and five buttons"
                );
                budget.total_components += row.populated();
            }
            Component::MediaGallery(gallery) => {
                anyhow::ensure!(
                    gallery.items.len() <= crate::page::limits::MEDIA_GALLERY_MAX_ITEMS,
                    "Media Gallery exceeds its item limit"
                );
                budget.total_components += gallery.items.len();
            }
            Component::Select(crate::page::select::SelectMenu::String(menu)) => {
                anyhow::ensure!(
                    menu.options.len() <= crate::page::limits::STRING_SELECT_MAX_OPTIONS,
                    "String Select exceeds its option limit"
                );
            }
            Component::Select(_)
            | Component::Separator(_)
            | Component::File(_)
            | Component::Button(_)
            | Component::LinkButton(_) => {}
        }
        Ok(budget)
    }

    let reserved = add(reserved_static, reserved_navigation);
    anyhow::ensure!(fits(reserved), "Static and navigation components exceed Discord message limits");

    let frame_id = frame_id.into();
    let mut pages: Vec<Vec<Element<T, D>>> = Vec::new();
    let mut current = Vec::new();
    let mut used = reserved;

    for element in elements {
        let cost = measure(&element.component)?;
        let combined_alone = add(reserved, cost);
        anyhow::ensure!(
            fits(combined_alone),
            "A single Frame element cannot fit within the remaining Discord message budget"
        );

        let combined = add(used, cost);
        if !fits(combined) && !current.is_empty() {
            pages.push(std::mem::take(&mut current));
            used = reserved;
        }
        used = add(used, cost);
        current.push(element);
    }
    if !current.is_empty() || pages.is_empty() {
        pages.push(current);
    }

    let state = PaginationState {
        frame_id,
        page_index: 0,
        page_count: pages.len(),
    };
    Ok(PaginationPlan { pages, state })
}

/// Complete request decoded from a framework-generated pagination control.
#[derive(Debug, Clone)]
pub struct PaginationRequest {
    pub custom_id: crate::app::store::CustomId,
    pub expected_state: serde_json::Value,
    pub frame_id: String,
    pub source_page_index: usize,
    pub target_page_index: usize,
    pub page_count: usize,
}
