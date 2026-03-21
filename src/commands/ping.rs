use crate::models::bot_data::{Context, Error};
use std::time::Instant;

/// Check the bot's latency and AniList API response time.
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let start = Instant::now();
    
    // Test AniList API response time with a simple query
    let data = ctx.data();
    let _ = crate::api::anilist::fetch_genres(&data.http_client, &data.cache, &data.rate_limiter).await?;
    let api_latency = start.elapsed().as_millis();
    
    let shard_id = ctx.serenity_context().shard_id;
    let manager = ctx.framework().shard_manager();
    let runners = manager.runners.lock().await;
    let heartbeat = runners.get(&shard_id)
        .and_then(|runner| runner.latency)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    ctx.say(format!(
        "🏓 **Pong!**\n\n**Heartbeat:** {}ms\n**AniList API:** {}ms",
        heartbeat, api_latency
    )).await?;

    Ok(())
}
