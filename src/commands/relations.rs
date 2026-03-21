use poise::CreateReply;
use crate::{
    api::anilist::fetch_media_by_title,
    models::bot_data::{Context, Error},
    utils::{
        embeds::relations_embed,
        errors::reply_error,
    },
};

/// Get relations (sequels, prequels, etc.) for a media title.
#[poise::command(slash_command, prefix_command)]
pub async fn relations(
    ctx: Context<'_>,
    #[description = "Media title to look up relations for"] title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;

    match fetch_media_by_title(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(media) => {
            ctx.send(CreateReply::default().embed(relations_embed(&media, prefs.title_language))).await?;
        }
        Err(e) => {
            tracing::warn!("Relations fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
