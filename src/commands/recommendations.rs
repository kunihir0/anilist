use poise::CreateReply;
use crate::{
    api::anilist::fetch_recommendations,
    models::bot_data::{Context, Error},
    utils::{
        embeds::recommendations_embed,
        errors::reply_error,
    },
};

/// Get top recommendations for a media title.
#[poise::command(slash_command, prefix_command)]
pub async fn recommendations(
    ctx: Context<'_>,
    #[description = "Media title to get recommendations for"] title: String,
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

    match fetch_recommendations(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(recommendations) => {
            ctx.send(CreateReply::default().embed(recommendations_embed(&recommendations, prefs.title_language, accent_color))).await?;
        }
        Err(e) => {
            tracing::warn!("Recommendations fetch failed for {title:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
