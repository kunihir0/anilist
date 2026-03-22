use crate::{
    api::anilist::fetch_studio,
    models::bot_data::{Context, Error},
    utils::{embeds::studio_embed, errors::reply_error},
};
use poise::CreateReply;

async fn autocomplete_studio(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    if partial.len() < 2 {
        return Vec::new().into_iter();
    }
    let data = ctx.data();
    match crate::api::anilist::fetch_studio_autocomplete(
        &data.http_client,
        &data.rate_limiter,
        partial,
    )
    .await
    {
        Ok(items) => items
            .into_iter()
            .map(|s| poise::serenity_prelude::AutocompleteChoice::new(
                s.name.clone(),
                s.name,
            ))
            .collect::<Vec<_>>()
            .into_iter(),
        Err(_) => Vec::new().into_iter(),
    }
}

/// Search AniList for a studio by name.
#[poise::command(slash_command, prefix_command, user_cooldown = 5, category = "Search")]
pub async fn studio(
    ctx: Context<'_>,
    #[description = "Studio name to search for"]
    #[autocomplete = "autocomplete_studio"]
    name: String,
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

    match fetch_studio(&data.http_client, &data.cache, &data.rate_limiter, &name).await {
        Ok(studio) => {
            ctx.send(CreateReply::default().embed(studio_embed(
                &studio,
                prefs.title_language,
                accent_color,
            )))
            .await?;
        }
        Err(e) => {
            tracing::warn!("Studio fetch failed for {name:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
