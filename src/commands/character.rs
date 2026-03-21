use poise::CreateReply;
use crate::{
    api::anilist::fetch_character,
    models::bot_data::{Context, Error},
    utils::{
        embeds::character_embed,
        errors::reply_error,
    },
};

/// Search AniList for a character by name.
#[poise::command(slash_command, prefix_command)]
pub async fn character(
    ctx: Context<'_>,
    #[description = "Character name to search for"] name: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;

    match fetch_character(&data.http_client, &data.cache, &data.rate_limiter, &name).await {
        Ok(character) => {
            ctx.send(CreateReply::default().embed(character_embed(&character, prefs.title_language))).await?;
        }
        Err(e) => {
            tracing::warn!("Character fetch failed for {name:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
