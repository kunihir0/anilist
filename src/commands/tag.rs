use crate::{
    api::anilist::fetch_by_tag,
    models::bot_data::{Context, Error, MediaType},
    utils::{embeds::media_list_embed, errors::reply_error},
};
use poise::{ChoiceParameter, CreateReply};

async fn autocomplete_tag(ctx: Context<'_>, partial: &str) -> impl Iterator<Item = String> {
    let tags = ctx.data().tags_cache.read().await;
    let partial = partial.to_lowercase();
    tags
        .iter()
        .filter(move |t| t.to_lowercase().contains(&partial))
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
}

/// Browse media by tag.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn tag(
    ctx: Context<'_>,
    #[description = "Tag to filter by"]
    #[autocomplete = "autocomplete_tag"]
    tag: String,
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

    match fetch_by_tag(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &tag,
        kind.as_str(),
    )
    .await
    {
        Ok(media) => {
            let title = format!("Top {} - {}", kind.name(), tag);
            ctx.send(CreateReply::default().embed(media_list_embed(
                &media,
                &title,
                prefs.title_language,
                accent_color,
            )))
            .await?;
        }
        Err(e) => {
            tracing::warn!("Tag fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
