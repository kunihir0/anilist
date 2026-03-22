use crate::{
    api::anilist::fetch_airing,
    models::bot_data::{Context, Error},
    utils::{embeds::airing_page_embed, errors::reply_error, pagination::paginate},
};

/// Show currently airing anime with episode countdowns.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn airing(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;
    let guild_id = ctx.guild_id().map(|id| id.get());
    let accent_color = if let Some(gid) = guild_id {
        data.store.get_settings(gid).await.accent_color
    } else {
        None
    };

    match fetch_airing(&data.http_client, &data.cache, &data.rate_limiter).await {
        Ok(shows) if shows.is_empty() => {
            ctx.say("No currently airing anime found.").await?;
        }
        Ok(shows) => {
            // Group shows into chunks of 5 for pagination
            let chunks: Vec<_> = shows.chunks(5).collect();
            let total_pages = chunks.len();
            let pages: Vec<_> = chunks
                .iter()
                .enumerate()
                .map(|(i, chunk)| {
                    airing_page_embed(
                        chunk,
                        i + 1,
                        total_pages,
                        prefs.title_language.clone(),
                        accent_color,
                    )
                })
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
