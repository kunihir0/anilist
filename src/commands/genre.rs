use poise::{ChoiceParameter, CreateReply};
use crate::{
    api::anilist::fetch_genre,
    models::bot_data::{Context, Error, MediaType},
    utils::{
        embeds::media_list_embed,
        errors::reply_error,
    },
};

async fn autocomplete_genre(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = String> {
    let genres = ctx.data().genres.read().await;
    let partial = partial.to_lowercase();
    genres.iter()
        .filter(move |g| g.to_lowercase().contains(&partial))
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
}

/// Browse media by genre.
#[poise::command(slash_command, prefix_command)]
pub async fn genre(
    ctx: Context<'_>,
    #[description = "Genre to filter by"]
    #[autocomplete = "autocomplete_genre"]
    genre: String,
    #[description = "Media type (ANIME or MANGA)"]
    media_type: Option<MediaType>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;

    let kind = media_type.unwrap_or(MediaType::Anime);

    match fetch_genre(&data.http_client, &data.cache, &data.rate_limiter, &genre, kind.as_str()).await {
        Ok(media) => {
            let title = format!("Top {} - {}", kind.name(), genre);
            ctx.send(CreateReply::default().embed(media_list_embed(&media, &title, prefs.title_language))).await?;
        }
        Err(e) => {
            tracing::warn!("Genre fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
