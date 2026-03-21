use poise::CreateReply;
use crate::{
    api::anilist::fetch_user,
    models::bot_data::{Context, Error},
    utils::{
        embeds::compare_embed,
        errors::reply_error,
    },
};

/// Compare two AniList user profiles side by side.
#[poise::command(slash_command, prefix_command)]
pub async fn compare(
    ctx: Context<'_>,
    #[description = "First AniList username"]  user1: String,
    #[description = "Second AniList username"] user2: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    // Fetch both users concurrently.
    let (res1, res2) = tokio::join!(
        fetch_user(&data.http_client, &data.cache, &data.rate_limiter, &user1),
        fetch_user(&data.http_client, &data.cache, &data.rate_limiter, &user2),
    );

    match (res1, res2) {
        (Ok(u1), Ok(u2)) => {
            ctx.send(CreateReply::default().embed(compare_embed(&u1, &u2))).await?;
        }
        (Err(e), _) => {
            reply_error(ctx, &e).await?;
        }
        (_, Err(e)) => {
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}