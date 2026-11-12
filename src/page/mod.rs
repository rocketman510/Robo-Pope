use crate::app::App;
use serenity::{all::{CreateContainer, CreateTextDisplay}, builder::{
    CreateComponent,
    CreateContainerComponent,
}, futures::future::BoxFuture};

#[derive(Debug)]
pub struct Page<T> {
    pub id: &'static str,
    pub dom: Vec<DomElement<T>>
}

#[derive(Debug)]
pub enum DomElement<T> {
    Text(TextElement),
    DynText(DynTextElement<T>),
}

#[derive(Debug)]
pub struct TextElement {
    pub content: &'static str,
}

pub struct DynTextElement<T> {
    pub content: Box<dyn Fn(&App, &T) -> BoxFuture<'static, String> + Send + Sync>,
}

impl<T> std::fmt::Debug for DynTextElement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DynTextElement")
            .field("content", &"<closure>")
            .finish()
    }
}

impl<T> Page<T> {
    pub async fn render(&self, app: &App, data: &T) -> CreateComponent<'_> {
        let mut components = Vec::new();

        for element in &self.dom {
            components.push(element.render(app, data).await);
        }

        CreateComponent::Container(
            CreateContainer::new(components)
        )
    }
    pub fn new(id: &'static str) -> Self {
        Self {id, dom: Vec::new()}
    }
    pub fn add_element(&mut self, element: DomElement<T>) -> &mut Self {
        self.dom.push(element);
        self
    }
}

trait Render<T> {
    async fn render(&self, app: &App, data: &T) -> CreateContainerComponent<'_>;
}

impl<T> Render<T> for TextElement {
    async fn render(&self, _: &App, _: &T) -> CreateContainerComponent<'_> {
        CreateContainerComponent::TextDisplay(
            CreateTextDisplay::new(self.content)
        )
    }
}

impl<T> Render<T> for DynTextElement<T> {
    async fn render(&self, app: &App, data: &T) -> CreateContainerComponent<'_> {
        let result = (self.content)(app, data).await;
        CreateContainerComponent::TextDisplay(
            CreateTextDisplay::new(result)
        )
    }
}

impl<T> Render<T> for DomElement<T> {
    async fn render(&self, app: &App, data: &T) -> CreateContainerComponent<'_> {
        match self {
            Self::Text(e) => {e.render(app, data).await}
            Self::DynText(e) => {e.render(app, data).await}
        }
    }
}
