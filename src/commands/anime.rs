use crate::{
    api::anilist::fetch_anime,
    models::bot_data::{Context, Error},
    utils::{
        embeds::media_embed,
        errors::reply_error,
        pagination::paginate,
    },
};

/// Search AniList for an anime by title.
#[poise::command(slash_command, prefix_command)]
pub async fn anime(
    ctx: Context<'_>,
    #[description = "Anime title to search for"] title: String,
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

    match fetch_anime(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(results) if results.is_empty() => {
            ctx.say(format!("No anime found for `{title}`.")).await?;
        }
        Ok(results) => {
            let pages: Vec<_> = results
                .iter()
                .map(|m| media_embed(m, "Anime", prefs.title_language.clone(), accent_color))
                .collect();
            paginate(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Anime fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
