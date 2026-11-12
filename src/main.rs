mod app;
mod page;
use std::sync::Arc;

use anyhow::{Context, Result};
use app::App;

use serenity::{
    prelude::*,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let token = Token::from_env("DISCORD_TOKEN").context("DISCORD_TOKEN is missing or invalid")?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;

    let mut app = App;

    let mut client = Client::builder(token, intents)
        .event_handler(Arc::new(app))
        .await
        .expect("Failed to create Discord client");

    println!("Starting bot...");

    if let Err(error) = client.start().await {
        eprintln!("Client error: {error}");
    }

    Ok(())
}
