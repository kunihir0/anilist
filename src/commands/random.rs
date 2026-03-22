use crate::{
    api::anilist::fetch_random,
    models::bot_data::{Context, Error, MediaType},
    utils::{embeds::media_embed, errors::reply_error},
};
use poise::{ChoiceParameter, CreateReply};

/// Get a random well-rated anime or manga from AniList.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn random(
    ctx: Context<'_>,
    #[description = "Anime or Manga"] media_type: Option<MediaType>,
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

    let kind = media_type.unwrap_or(MediaType::Anime);

    match fetch_random(&data.http_client, &data.rate_limiter, kind.as_str(), None).await {
        Ok(media) => {
            ctx.send(CreateReply::default().embed(media_embed(
                &media,
                kind.name(),
                prefs.title_language,
                accent_color,
                prefs.compact_mode,
            )))
            .await?;
        }
        Err(e) => {
            tracing::warn!("Random fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
