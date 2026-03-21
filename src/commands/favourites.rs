use poise::CreateReply;
use crate::{
    api::anilist::fetch_favourites,
    models::bot_data::{Context, Error},
    utils::{
        embeds::favourites_embed,
        errors::reply_error,
    },
};

/// Look up a user's public AniList favourites.
#[poise::command(slash_command, prefix_command)]
pub async fn favourites(
    ctx: Context<'_>,
    #[description = "AniList username"] username: String,
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

    match fetch_favourites(&data.http_client, &data.cache, &data.rate_limiter, &username).await {
        Ok(user) => {
            ctx.send(CreateReply::default().embed(favourites_embed(&user, prefs.title_language, accent_color))).await?;
        }
        Err(e) => {
            tracing::warn!("Favourites fetch failed for {username:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
