use crate::models::bot_data::{Context, Error};
use poise::serenity_prelude as serenity;

/// Show bot statistics and diagnostics.
#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();

    // Uptime
    let elapsed = data.start_time.elapsed();
    let days = elapsed.as_secs() / 86400;
    let hours = (elapsed.as_secs() % 86400) / 3600;
    let minutes = (elapsed.as_secs() % 3600) / 60;
    let uptime = match (days, hours) {
        (0, 0) => format!("{}m", minutes),
        (0, h) => format!("{}h {}m", h, minutes),
        (d, h) => format!("{}d {}h {}m", d, h, minutes),
    };

    // Server count
    let guild_count = ctx.cache().guild_count();

    // Cache entries
    let cache_entries = data.cache.entry_count();

    // Latency
    let ping = ctx.ping().await;
    let latency = format!("{}ms", ping.as_millis());

    let embed = serenity::CreateEmbed::new()
        .title("📊 Bot Statistics")
        .colour(serenity::Colour::new(0x2ECC71))
        .timestamp(serenity::Timestamp::now())
        .field("Uptime", &uptime, true)
        .field("Servers", guild_count.to_string(), true)
        .field("Latency", &latency, true)
        .field("Cache Entries", cache_entries.to_string(), true)
        .field("Rust Version", env!("CARGO_PKG_VERSION"), true);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}
