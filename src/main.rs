// Runnable framework showcase and Discord Gateway startup.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

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
    app::{store::{SqliteStore, Store}, App, Ctx, RenderCtx},
    page::{
        Button, ButtonStyle, Container, Element, LinkButton, MediaGallery, Modal,
        Page, Section, SelectOption, Separator, StringSelect, TextInputStyle,
        Thumbnail,
    },
};

#[derive(Clone, Default)]
struct DemoDatabase {
    button_clicks: Arc<AtomicU64>,
    selections: Arc<AtomicU64>,
    modal_submissions: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DemoData {
    username: String,
    visits: u64,
}

#[derive(Debug, Deserialize)]
struct DemoForm {
    message: String,
}

fn register_demo_page(app: &App<DemoDatabase>) -> anyhow::Result<()> {
    let page = Page::<DemoData, DemoDatabase>::new("framework-demo")
        .add_text("# Robo-Pope framework showcase")
        .add_text(|ctx: RenderCtx<DemoDatabase>, data: Arc<DemoData>| async move {
            let clicks = ctx.app.db.button_clicks.load(Ordering::Relaxed);
            let selections = ctx.app.db.selections.load(Ordering::Relaxed);
            let submissions = ctx.app.db.modal_submissions.load(Ordering::Relaxed);

            Ok(format!(
                "Welcome, {}. Visits: {}.\n\nButton clicks: {} · Selections: {} · Form submissions: {}",
                data.username, data.visits, clicks, selections, submissions
            ))
        })
        .add_section(
            Section::new()
                .set_text(
                    "Section component",
                    "Three independently configurable text slots.",
                    None,
                )
                .set_accessory(
                    Thumbnail::new("https://cdn.discordapp.com/embed/avatars/0.png")
                        .set_description("Discord default avatar"),
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
                "Refresh stats",
                "demo-refresh",
                ButtonStyle::Primary,
                None,
                false,
                |ctx: Ctx<DemoDatabase>, data: DemoData| async move {
                    ctx.app.db.button_clicks.fetch_add(1, Ordering::Relaxed);

                    let updated = ctx
                        .app
                        .pages
                        .query::<DemoData>("framework-demo")?
                        .render(&ctx.app, data)
                        .await?;

                    ctx.edit_message(updated).await?;
                    Ok(())
                },
            ),
            Button::new(
                "Open demo form",
                "demo-open-form",
                ButtonStyle::Secondary,
                None,
                false,
                |ctx, data: DemoData| async move {
                    let modal = ctx
                        .app
                        .modals
                        .query::<DemoData>("framework-demo-modal")?
                        .render(&ctx.app, data)
                        .await?;

                    ctx.open_modal(modal).await?;
                    Ok(())
                },
            ),
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
            |ctx: Ctx<DemoDatabase>, _data: DemoData, selected: Vec<String>| async move {
                ctx.app.db.selections.fetch_add(1, Ordering::Relaxed);
                ctx.reply(format!("Selected: {}", selected.join(", ")))
                    .await?;
                Ok(())
            },
        ))
        .add_gallery(
            MediaGallery::new()
                .add_media("https://cdn.discordapp.com/embed/avatars/0.png")
                .add_media("https://cdn.discordapp.com/embed/avatars/1.png"),
        )
        .add_frame(|ctx: RenderCtx<DemoDatabase>, _data: Arc<DemoData>| async move {
            let clicks = ctx.app.db.button_clicks.load(Ordering::Relaxed);
            let selections = ctx.app.db.selections.load(Ordering::Relaxed);
            let submissions = ctx.app.db.modal_submissions.load(Ordering::Relaxed);

            Ok(vec![Element::text(format!(
                "**Live application state**\nButton clicks: {clicks}\nSelection events: {selections}\nModal submissions: {submissions}"
            ))])
        });

    page.register(app)?;
    println!("Registered demo Page: framework-demo");

    Ok(())
}

fn register_demo_modal(app: &App<DemoDatabase>) -> anyhow::Result<()> {
    let modal = Modal::<DemoData, DemoDatabase>::new("framework-demo-modal")
        .set_title("Framework Demo Form")
        .add_text_input("message", "Your message", TextInputStyle::Short)
        .on_submit_typed(|ctx: Ctx<DemoDatabase>, _data: DemoData, form: DemoForm| async move {
            ctx.app
                .db
                .modal_submissions
                .fetch_add(1, Ordering::Relaxed);

            ctx.reply(format!("Modal submitted: {}", form.message))
                .await?;
            Ok(())
        });

    modal.register(app)?;
    println!("Registered demo Modal: framework-demo-modal");

    Ok(())
}

struct DemoHandler {
    app: App<DemoDatabase>,
}

impl DemoHandler {
    fn new(app: App<DemoDatabase>) -> Self {
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
                if new_message.author.bot() || new_message.content.trim() != "!test" {
                    return;
                }

                let data = DemoData {
                    username: new_message.author.name.to_string(),
                    visits: 1,
                };

                let page = match self.app.pages.query::<DemoData>("framework-demo") {
                    Ok(page) => page,
                    Err(error) => {
                        eprintln!("Failed to query demo Page: {error:#}");
                        return;
                    }
                };

                let rendered = match page.render(&self.app, data).await {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        eprintln!("Failed to render demo Page: {error:#}");
                        return;
                    }
                };

                let response = CreateMessage::new()
                    .components(rendered.into_components())
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

            FullEvent::InteractionCreate { interaction, .. } => {
                if let Err(error) = self.app.dispatch_interaction(interaction.clone()).await {
                    eprintln!("Failed to dispatch demo interaction: {error:#}");
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

    // The demo's host database remains a concrete App<D> type. The routing
    // Store is separate and persists the four-field interaction records.
    std::fs::create_dir_all(".robo-pope")
        .context("Failed to create the framework data directory")?;
    let database = DemoDatabase::default();
    let store: Arc<dyn Store> =
        Arc::new(SqliteStore::open(".robo-pope/routing.sqlite").await?);
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
    println!("The Message Content privileged intent must be enabled for !test.");
    client
        .start()
        .await
        .context("Discord Gateway connection failed")?;

    Ok(())
}
