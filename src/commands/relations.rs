use crate::{
    api::anilist::fetch_media_by_title,
    models::bot_data::{Context, Error},
    utils::{embeds::relations_embed, errors::reply_error},
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

/// Get relations (sequels, prequels, etc.) for a media title.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn relations(
    ctx: Context<'_>,
    #[description = "Media title to look up relations for"]
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

    match fetch_media_by_title(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(media) => {
            ctx.send(CreateReply::default().embed(relations_embed(
                &media,
                prefs.title_language,
                accent_color,
            )))
            .await?;
        }
        Err(e) => {
            tracing::warn!("Relations fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
