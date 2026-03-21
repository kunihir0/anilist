use poise::{ChoiceParameter, CreateReply};
use crate::{
    api::anilist::fetch_trending,
    models::bot_data::{Context, Error, MediaType},
    utils::{
        embeds::media_list_embed,
        errors::reply_error,
    },
};

/// See what's currently trending on AniList.
#[poise::command(slash_command, prefix_command)]
pub async fn trending(
    ctx: Context<'_>,
    #[description = "Media type (ANIME or MANGA)"]
    media_type: Option<MediaType>,
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

    match fetch_trending(&data.http_client, &data.cache, &data.rate_limiter, kind.as_str()).await {
        Ok(media) => {
            let title = format!("Trending {}", kind.name());
            ctx.send(CreateReply::default().embed(media_list_embed(&media, &title, prefs.title_language, accent_color))).await?;
        }
        Err(e) => {
            tracing::warn!("Trending fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
