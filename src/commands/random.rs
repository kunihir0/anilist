use poise::{ChoiceParameter, CreateReply};
use crate::{
    api::anilist::fetch_random,
    models::bot_data::{Context, Error, MediaType},
    utils::{
        embeds::media_embed,
        errors::reply_error,
    },
};

/// Get a random well-rated anime or manga from AniList.
#[poise::command(slash_command, prefix_command)]
pub async fn random(
    ctx: Context<'_>,
    #[description = "Anime or Manga"] media_type: Option<MediaType>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;

    let kind = media_type.unwrap_or(MediaType::Anime);

    match fetch_random(&data.http_client, &data.rate_limiter, kind.as_str()).await {
        Ok(media) => {
            ctx.send(CreateReply::default().embed(media_embed(&media, kind.name(), prefs.title_language))).await?;
        }
        Err(e) => {
            tracing::warn!("Random fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
