use crate::{
    api::anilist::fetch_recommendations,
    models::bot_data::{Context, Error},
    utils::{embeds::recommendations_embed, errors::reply_error},
};
use poise::CreateReply;

async fn autocomplete_media(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    if partial.len() < 2 {
        return Vec::new().into_iter();
    }
    let data = ctx.data();
    // Default to anime for relation/recommendation search since we don't know the exact type
    match crate::api::anilist::fetch_media_autocomplete(
        &data.http_client,
        &data.rate_limiter,
        partial,
        "ANIME",
    )
    .await
    {
        Ok(items) => items
            .into_iter()
            .map(|m| {
                let label = match m.format.as_deref() {
                    Some(fmt) => format!("{} ({})", m.title.preferred(), fmt),
                    None => m.title.preferred().to_string(),
                };
                poise::serenity_prelude::AutocompleteChoice::new(
                    label,
                    m.title.preferred().to_string(),
                )
            })
            .collect::<Vec<_>>()
            .into_iter(),
        Err(_) => Vec::new().into_iter(),
    }
}

/// Get top recommendations for a media title.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn recommendations(
    ctx: Context<'_>,
    #[description = "Media title to get recommendations for"]
    #[autocomplete = "autocomplete_media"]
    title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;
    let guild_id = ctx.guild_id().map(|id| id.get());
    let accent_color = if let Some(gid) = guild_id {
        data.store.get_settings(gid).await.accent_color
    } else {
        None
    };

    match fetch_recommendations(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(recommendations) => {
            ctx.send(CreateReply::default().embed(recommendations_embed(
                &recommendations,
                prefs.title_language,
                accent_color,
            )))
            .await?;
        }
        Err(e) => {
            tracing::warn!("Recommendations fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
