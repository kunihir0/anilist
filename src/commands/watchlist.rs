use poise::{ChoiceParameter, CreateReply};
use crate::{
    api::anilist::fetch_watchlist,
    models::bot_data::{Context, Error, MediaType},
    utils::{
        embeds::watchlist_embed,
        errors::reply_error,
    },
};

/// Look up a user's media watchlist collection.
#[poise::command(slash_command, prefix_command)]
pub async fn watchlist(
    ctx: Context<'_>,
    #[description = "AniList username"] username: String,
    #[description = "Media type (ANIME or MANGA)"] media_type: Option<MediaType>,
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

    match fetch_watchlist(&data.http_client, &data.cache, &data.rate_limiter, &username, kind.as_str()).await {
        Ok(collection) => {
            ctx.send(CreateReply::default().embed(watchlist_embed(&collection, &username, kind.name(), prefs.title_language, accent_color))).await?;
        }
        Err(e) => {
            tracing::warn!("Watchlist fetch failed for {username:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
