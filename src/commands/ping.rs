use crate::models::bot_data::{Context, Error};
use poise::serenity_prelude::{self as serenity, CreateEmbed};
use std::time::Instant;

/// Check the bot's latency and AniList API response time.
#[poise::command(slash_command, prefix_command, ephemeral, category = "Utility")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();

    // Test AniList API response time with a simple query
    let data = ctx.data();
    let _ = crate::api::anilist::fetch_genres(&data.http_client, &data.cache, &data.rate_limiter)
        .await?;
    let api_latency = start.elapsed().as_millis();

    let shard_id = ctx.serenity_context().shard_id;
    let manager = ctx.framework().shard_manager();
    let runners = manager.runners.lock().await;
    let heartbeat = runners
        .get(&shard_id)
        .and_then(|runner| runner.latency)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let heartbeat_indicator = match heartbeat {
        0..=100 => "🟢",
        101..=250 => "🟡",
        _ => "🔴",
    };
    let api_indicator = match api_latency {
        0..=200 => "🟢",
        201..=500 => "🟡",
        _ => "🔴",
    };

    let embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .colour(serenity::Colour::new(0x02a9ff))
        .field(
            "Gateway Heartbeat",
            format!("{heartbeat_indicator} **{heartbeat}**ms"),
            true,
        )
        .field(
            "AniList API",
            format!("{api_indicator} **{api_latency}**ms"),
            true,
        )
        .timestamp(serenity::Timestamp::now())
        .footer(serenity::CreateEmbedFooter::new("Powered by AniList"));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}
