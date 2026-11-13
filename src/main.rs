//! Runnable framework showcase and Discord Gateway startup.
use std::sync::Arc;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{GatewayIntents, Http, Token},
    Client,
};

use robo_pope::{
    app::{handler::AppHandler, store::MemoryStore, App, Database},
    page::{
        dynamic, Button, ButtonStyle, Container, Element, Frame, LinkButton,
        MediaGallery, Modal, Page, Section, SelectOption, Separator, StringSelect,
        TextInputStyle, Thumbnail,
    },
};

#[derive(Debug, Serialize, Deserialize)]
struct DemoData {
    username: String,
    visits: u64,
}

#[derive(Debug, Deserialize)]
struct DemoForm {
    message: String,
}

fn register_demo_page(app: &App) -> anyhow::Result<()> {
    let page = Page::<DemoData>::new("framework-demo")
        .add_text("# Robo-Pope framework showcase")
        .add_text(dynamic::<String, DemoData, _, _>(|_ctx, data| async move {
            Ok(format!("Welcome, {}. Visits: {}", data.username, data.visits))
        }))
        .add_section(
            Section::new()
                .set_text(
                    "Section component",
                    "Three independently configurable text slots.",
                    None,
                )
                .set_accessory(
                    Thumbnail::new("https://example.com/avatar.png")
                        .set_description("Example thumbnail"),
                ),
        )
        .add_container(
            Container::new()
                .set_accent_color(0x5865F2)
                .add_text("## Container component")
                .add_separator(Separator::new())
                .add_text("Container children retain their declared order."),
        )
        .add_button_row((
            Some(Button::new(
                "Test button",
                "demo-button",
                ButtonStyle::Primary,
                |_ctx, _data: DemoData| async move {
                    println!("The demo button callback was invoked.");
                    Ok(())
                },
            )),
            None,
            None,
            None,
            None,
        ))
        .add_link_button(LinkButton::new(
            "Robo-Pope repository",
            "https://github.com/rocketman510/Robo-Pope",
        ))
        .add_selection_menu(StringSelect::new(
            "Choose an option",
            "demo-select",
            [
                SelectOption::new("First option", "first"),
                SelectOption::new("Second option", "second"),
                SelectOption::new("Third option", "third"),
            ],
            |_ctx, _data: DemoData, selected: Vec<String>| async move {
                println!("Selected values: {selected:?}");
                Ok(())
            },
        ))
        .add_gallery(
            MediaGallery::new()
                .add_media("https://example.com/first-image.png")
                .add_media("https://example.com/second-image.png"),
        )
        .add_frame(
            Frame::<DemoData>::new("demo-frame", |_ctx, data| async move {
                Ok(vec![
                    Element::text(format!("Generated Frame content for {}.", data.username))
                        .keyed("demo-frame-entry"),
                ])
            })
            .paginate(),
        )
        .paginate();

    let _registered_page = page.register(app)?;
    println!("Registered demo Page: framework-demo");
    Ok(())
}

fn register_demo_modal(app: &App) -> anyhow::Result<()> {
    let modal = Modal::<DemoData>::new("framework-demo-modal")
        .set_title("Framework Demo Form")
        .add_text_input("message", "Your message", TextInputStyle::Short)
        .on_submit_typed(
            |_ctx, _data, form: DemoForm| async move {
                println!("Modal submitted: {}", form.message);
                Ok(())
            },
        );

    let _registered_modal = modal.register(app)?;
    println!("Registered demo Modal: framework-demo-modal");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    // Serenity Next expects its validated Token type, not a String.
    let token = Token::from_env("DISCORD_TOKEN")
        .context("Could not load a valid DISCORD_TOKEN from .env")?;

    let http = Arc::new(Http::new(token.clone()));
    let database = Database::new(Arc::new(MemoryStore::default()));
    let app = App::new(http, database);

    register_demo_page(&app)?;
    register_demo_modal(&app)?;

    let mut client = Client::builder(token, GatewayIntents::GUILDS)
        .event_handler(Arc::new(AppHandler::new(app)))
        .await
        .context("Failed to create the Serenity client")?;

    println!("Connecting Robo-Pope to Discord...");
    client.start().await.context("Discord Gateway connection failed")?;

    Ok(())
}
