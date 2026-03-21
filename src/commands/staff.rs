use poise::CreateReply;
use crate::{
    api::anilist::fetch_staff,
    models::bot_data::{Context, Error},
    utils::{
        embeds::staff_embed,
        errors::reply_error,
    },
};

/// Search AniList for a staff member (VA, director, etc) by name.
#[poise::command(slash_command, prefix_command)]
pub async fn staff(
    ctx: Context<'_>,
    #[description = "Staff name to search for"] name: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_staff(&data.http_client, &data.cache, &data.rate_limiter, &name).await {
        Ok(staff) => {
            ctx.send(CreateReply::default().embed(staff_embed(&staff))).await?;
        }
        Err(e) => {
            tracing::warn!("Staff fetch failed for {name:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}