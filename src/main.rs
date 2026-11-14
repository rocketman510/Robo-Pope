// Runnable framework showcase and Discord Gateway startup.

use std::sync::Arc;

use anyhow::Context as _;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{
        Context, CreateMessage, EventHandler, FullEvent, GatewayIntents, Http,
        MessageFlags, Token,
    },
    async_trait,
    Client,
};

use robo_pope::{
    app::{store::{FileStore, Store}, App, Database, RenderCtx},
    page::{
        Button, ButtonStyle, Container, Element, LinkButton,
        MediaGallery, Section, SelectOption, Separator, StringSelect,
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

fn register_demo_page(app: &App<Database>) -> anyhow::Result<()> {
    let page = app.page::<DemoData>("framework-demo")
        .add_text("# Robo-Pope framework showcase")
        .add_text(|_ctx: RenderCtx<Database>, data: Arc<DemoData>| async move {
            Ok(format!("Welcome, {}. Visits: {}", data.username, data.visits))
        })
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
            Button::new(
                "Test button",
                "demo-button",
                ButtonStyle::Primary,
                None,
                false,
                |_ctx, _data: DemoData| async move {
                    println!("The demo button callback was invoked.");
                    Ok(())
                },
            ),
            None,
            None,
            None,
            None,
        ))
        .add_link_button(LinkButton::new(
            "Robo-Pope repository",
            "https://github.com/rocketman510/Robo-Pope",
            None,
            false,
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
        .add_frame(|_ctx, data| async move {
            Ok(vec![
                Element::text(format!(
                    "Generated Frame content for {}.",
                    data.username
                )),
            ])
        });

    page.register(app)?;
    println!("Registered demo Page: framework-demo");

    Ok(())
}

fn register_demo_modal(app: &App<Database>) -> anyhow::Result<()> {
    let modal = app.modal::<DemoData>("framework-demo-modal")
        .set_title("Framework Demo Form")
        .add_text_input("message", "Your message", TextInputStyle::Short)
        .on_submit_typed(
            |_ctx, _data, form: DemoForm| async move {
                println!("Modal submitted: {}", form.message);
                Ok(())
            },
        );

    modal.register(app)?;
    println!("Registered demo Modal: framework-demo-modal");

    Ok(())
}

struct DemoHandler {
    app: App<Database>,
}

impl DemoHandler {
    fn new(app: App<Database>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl EventHandler for DemoHandler {
    async fn dispatch(&self, ctx: &Context, event: &FullEvent) {
        match event {
            FullEvent::Ready { data_about_bot, .. } => {
                println!("{} is connected.", data_about_bot.user.name);
                println!("Send !test in a channel I can read.");
            }

            FullEvent::Message { new_message, .. } => {
                if new_message.author.bot() {
                    return;
                }

                if new_message.content.trim() != "!test" {
                    return;
                }

                let data = DemoData {
                    username: new_message.author.name.to_string(),
                    visits: 1,
                };

                // Retrieve the immutable registered Page definition.
                let page = match self.app.pages.query("framework-demo") {
                    Ok(page) => page,
                    Err(error) => {
                        eprintln!("Failed to query demo Page: {error:#}");
                        return;
                    }
                };

                // Render the registered Page with this message's data.
                let rendered = match page.render(&self.app, data).await {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        eprintln!("Failed to render demo Page: {error:#}");
                        return;
                    }
                };

                let components = rendered.into_components();

                // Send the rendered Components V2 message as a reply.
                let response = CreateMessage::new()
                    .components(components)
                    .flags(MessageFlags::from_bits_retain(1 << 15))
                    .reference_message(new_message);

                if let Err(error) = new_message
                    .channel_id
                    .send_message(&ctx.http, response)
                    .await
                {
                    eprintln!("Failed to send demo Page: {error}");
                } else {
                    println!("Demo Page sent successfully.");
                }
            }

            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let token = Token::from_env("DISCORD_TOKEN")
        .context("Could not load a valid DISCORD_TOKEN from .env")?;

    let http = Arc::new(Http::new(token.clone()));
    // Persist interaction references so custom IDs remain routable after a restart.
    let database = Database::new(());
    let store: Arc<dyn Store> = Arc::new(FileStore::open(".robo-pope/store.json")?);
    let app = App::new(http, database, store);

    register_demo_page(&app)?;
    register_demo_modal(&app)?;

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(DemoHandler::new(app)))
        .await
        .context("Failed to create the Serenity client")?;

    println!("Connecting Robo-Pope to Discord...");
    client.start().await.context("Discord Gateway connection failed")?;

    Ok(())
}


