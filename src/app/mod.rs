use serenity::{all::{Context, CreateMessage, EventHandler, FullEvent, MessageFlags }, async_trait};

use crate::page::{DomElement, Page, TextElement};

#[derive(Debug)]
pub struct App;

#[async_trait]
impl EventHandler for App {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Message { new_message, .. } => {
                println!("{:#?}", new_message.content);
                let mut page = Page::<&'static str>::new("Test");

                page.add_element(DomElement::Text(TextElement { content: "Test" }));

                let render = page.render(self, &"test").await;

                let message = CreateMessage::new()
                    .reference_message(new_message)
                    .components(vec![render])
                    .flags(MessageFlags::from_bits_retain(1 << 15));

                if let Err(error) = new_message
                    .channel_id
                    .send_message(&ctx.http, message)
                    .await
                {
                    eprintln!("Failed to send message: {error}");
                }            
            },
            FullEvent::InteractionCreate { interaction, .. } => {
                println!("Received an interaction: {interaction:#?}");
            }
            _ => {}
        }
    }
}
