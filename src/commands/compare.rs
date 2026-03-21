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
    #[description = "First username"] user1: String,
    #[description = "Second username"] user2: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let guild_id = ctx.guild_id().map(|id| id.get());
    let accent_color = if let Some(gid) = guild_id {
        data.store.get_settings(gid).await.accent_color
    } else {
        None
    };

    let u1_task = fetch_user(&data.http_client, &data.cache, &data.rate_limiter, &user1);
    let u2_task = fetch_user(&data.http_client, &data.cache, &data.rate_limiter, &user2);

    match tokio::try_join!(u1_task, u2_task) {
        Ok((u1, u2)) => {
            ctx.send(CreateReply::default().embed(compare_embed(&u1, &u2, accent_color))).await?;
        }
        Err(e) => {
            tracing::warn!("Comparison fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
