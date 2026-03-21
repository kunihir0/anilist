use poise::CreateReply;
use crate::{
    api::anilist::fetch_studio,
    models::bot_data::{Context, Error},
    utils::{
        embeds::studio_embed,
        errors::reply_error,
    },
};

/// Look up an anime studio and display their notable works.
#[poise::command(slash_command, prefix_command)]
pub async fn studio(
    ctx: Context<'_>,
    #[description = "Studio name to search for"] name: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_studio(&data.http_client, &data.cache, &data.rate_limiter, &name).await {
        Ok(studio) => {
            ctx.send(CreateReply::default().embed(studio_embed(&studio))).await?;
        }
        Err(e) => {
            tracing::warn!("Studio fetch failed for {name:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}