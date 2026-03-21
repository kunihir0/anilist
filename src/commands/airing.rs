use poise::CreateReply;
use crate::{
    api::anilist::fetch_airing,
    models::bot_data::{Context, Error},
    utils::{
        embeds::airing_page_embed,
        errors::{not_found_embed, reply_error},
        pagination::paginate,
    },
};

/// Show currently airing anime with episode countdown timers.
#[poise::command(slash_command, prefix_command)]
pub async fn airing(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_airing(&data.http_client, &data.cache, &data.rate_limiter).await {
        Ok(shows) if shows.is_empty() => {
            ctx.send(
                CreateReply::default()
                    .embed(not_found_embed("Airing Anime", "currently"))
                    .ephemeral(true),
            )
            .await?;
        }
        Ok(shows) => {
            // 5 shows per page — inline fields look cramped beyond that
            let chunks: Vec<_> = shows.chunks(5).collect();
            let total_pages = chunks.len();
            let pages: Vec<_> = chunks
                .iter()
                .enumerate()
                .map(|(i, chunk)| airing_page_embed(chunk, i + 1, total_pages))
                .collect();
            paginate(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Airing fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}