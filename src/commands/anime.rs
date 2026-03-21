use poise::CreateReply;
use crate::{
    api::anilist::fetch_anime,
    models::bot_data::{Context, Error},
    utils::{
        embeds::media_embed,
        errors::{not_found_embed, reply_error},
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

    match fetch_anime(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(results) if results.is_empty() => {
            ctx.send(
                CreateReply::default()
                    .embed(not_found_embed("Anime", &title))
                    .ephemeral(true),
            )
            .await?;
        }
        Ok(results) => {
            let pages: Vec<_> = results.iter().map(|m| media_embed(m, "Anime")).collect();
            paginate(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Anime fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}