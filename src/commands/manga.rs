use poise::CreateReply;
use crate::{
    api::anilist::fetch_manga,
    models::bot_data::{Context, Error},
    utils::{
        embeds::media_embed,
        errors::reply_error,
        pagination::paginate,
    },
};

/// Search AniList for a manga by title.
#[poise::command(slash_command, prefix_command)]
pub async fn manga(
    ctx: Context<'_>,
    #[description = "Manga title to search for"] title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;

    match fetch_manga(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(results) if results.is_empty() => {
            ctx.say(format!("No manga found for `{title}`.")).await?;
        }
        Ok(results) => {
            let pages: Vec<_> = results
                .iter()
                .map(|m| media_embed(m, "Manga", prefs.title_language.clone()))
                .collect();
            paginate(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Manga fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
