//! Verifies that the Discord bot token in .env is valid.
//!
//! Run with: cargo run --example discord_token_check

use anyhow::Context;
use serenity::all::Http;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN")
        .context("DISCORD_TOKEN is missing; add it to the project's .env file")?;

    let http = Http::new(&token);
    let bot = http
        .get_current_user()
        .await
        .context("Discord rejected the token or the API request failed")?;

    println!("Discord authentication succeeded.");
    println!("Bot: {} ({})", bot.name, bot.id);

    Ok(())
}
