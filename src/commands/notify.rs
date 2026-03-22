use crate::{
    api::anilist::fetch_anime,
    models::bot_data::{Context, Error},
    store::AiringSubscription,
};

/// Manage your airing notification subscriptions.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("subscribe", "unsubscribe", "list"),
    category = "Utility"
)]
pub async fn notify(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Subscribe to airing notifications for an anime.
#[poise::command(slash_command, prefix_command)]
pub async fn subscribe(
    ctx: Context<'_>,
    #[description = "Anime title to subscribe to"]
    #[autocomplete = "crate::commands::autocomplete_anime"]
    title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let user_id = ctx.author().id.get();

    // Search for the anime to get a media_id
    let results = fetch_anime(&data.http_client, &data.cache, &data.rate_limiter, &title).await?;

    let media = results
        .into_iter()
        .next()
        .ok_or("No anime found with that title.")?;

    let channel_id = ctx.channel_id().get();
    let guild_id = ctx.guild_id().map(|g| g.get());

    let sub = AiringSubscription {
        id: format!("{}-{}-{}", user_id, media.id, channel_id),
        user_id,
        guild_id,
        channel_id: Some(channel_id),
        media_id: media.id as u64,
        title: media.title.preferred().to_string(),
    };

    match data.store.add_airing_subscription(sub).await {
        Ok(()) => {
            ctx.say(format!(
                "✅ Subscribed to airing notifications for **{}** in this channel.",
                media.title.preferred()
            ))
            .await?;
        }
        Err(e) => {
            if e.to_string().contains("UNIQUE") {
                ctx.say("You're already subscribed to this anime in this channel.")
                    .await?;
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Unsubscribe from airing notifications for an anime.
#[poise::command(slash_command, prefix_command)]
pub async fn unsubscribe(
    ctx: Context<'_>,
    #[description = "Anime title to unsubscribe from"]
    #[autocomplete = "crate::commands::autocomplete_anime"]
    title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let user_id = ctx.author().id.get();

    let results = fetch_anime(&data.http_client, &data.cache, &data.rate_limiter, &title).await?;

    let media = results
        .into_iter()
        .next()
        .ok_or("No anime found with that title.")?;

    let channel_id = Some(ctx.channel_id().get());

    let removed = data
        .store
        .remove_airing_subscription(user_id, media.id as u64, channel_id)
        .await?;

    if removed {
        ctx.say(format!(
            "🔕 Unsubscribed from airing notifications for **{}**.",
            media.title.preferred()
        ))
        .await?;
    } else {
        ctx.say("You don't have an active subscription for that anime in this channel.")
            .await?;
    }

    Ok(())
}

/// List your active airing subscriptions.
#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let user_id = ctx.author().id.get();

    let subs = data.store.get_user_subscriptions(user_id).await;

    if subs.is_empty() {
        ctx.say("You have no active airing subscriptions. Use `/notify subscribe` to add one!")
            .await?;
    } else {
        let list: String = subs
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let channel = s
                    .channel_id
                    .map(|c| format!("<#{}>", c))
                    .unwrap_or_else(|| "DM".to_string());
                format!("{}. **{}** → {}", i + 1, s.title, channel)
            })
            .collect::<Vec<_>>()
            .join("\n");

        ctx.say(format!("📋 **Your Airing Subscriptions:**\n{}", list))
            .await?;
    }

    Ok(())
}
