use crate::{
    api::anilist::fetch_user,
    models::bot_data::{Context, Error},
    utils::{embeds::user_embed, errors::reply_error},
};
use poise::CreateReply;

/// Look up a public AniList user profile and display their stats.
#[poise::command(slash_command, prefix_command)]
pub async fn profile(
    ctx: Context<'_>,
    #[description = "AniList username to look up"] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|id| id.get());
    let accent_color = if let Some(gid) = guild_id {
        data.store.get_settings(gid).await.accent_color
    } else {
        None
    };

    match fetch_user(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &username,
    )
    .await
    {
        Ok(user) => {
            ctx.send(CreateReply::default().embed(user_embed(&user, accent_color)))
                .await?;
        }
        Err(e) => {
            tracing::warn!("User fetch failed for {username:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
