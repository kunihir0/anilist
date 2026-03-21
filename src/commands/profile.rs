use poise::CreateReply;
use crate::{
    api::anilist::fetch_user,
    models::bot_data::{Context, Error},
    utils::{
        embeds::user_embed,
        errors::reply_error,
    },
};

/// Look up a public AniList user profile and display their stats.
#[poise::command(slash_command, prefix_command)]
pub async fn profile(
    ctx: Context<'_>,
    #[description = "AniList username to look up"] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_user(&data.http_client, &data.cache, &data.rate_limiter, &username).await {
        Ok(user) => {
            ctx.send(CreateReply::default().embed(user_embed(&user))).await?;
        }
        Err(e) => {
            tracing::warn!("User fetch failed for {username:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}