use poise::{ChoiceParameter, CreateReply};
use crate::{
    api::anilist::fetch_genre,
    models::bot_data::{Context, Error, MediaType},
    utils::{
        embeds::media_list_embed,
        errors::reply_error,
    },
};

#[derive(Debug, poise::ChoiceParameter, Clone, Copy, PartialEq, Eq)]
pub enum Genre {
    Action,
    Adventure,
    Comedy,
    Drama,
    Ecchi,
    Fantasy,
    Horror,
    #[name = "Mahou Shoujo"]
    MahouShoujo,
    Mecha,
    Music,
    Mystery,
    Psychological,
    Romance,
    #[name = "Sci-Fi"]
    SciFi,
    #[name = "Slice of Life"]
    SliceOfLife,
    Sports,
    Supernatural,
    Thriller,
}

/// Browse media by genre.
#[poise::command(slash_command, prefix_command)]
pub async fn genre(
    ctx: Context<'_>,
    #[description = "Genre to filter by"]
    genre: Genre,
    #[description = "Media type (ANIME or MANGA)"]
    media_type: Option<MediaType>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    let kind = media_type.unwrap_or(MediaType::Anime);

    match fetch_genre(&data.http_client, &data.cache, &data.rate_limiter, genre.name(), kind.as_str()).await {
        Ok(media) => {
            let title = format!("Top {} - {}", kind.name(), genre.name());
            ctx.send(CreateReply::default().embed(media_list_embed(&media, &title))).await?;
        }
        Err(e) => {
            tracing::warn!("Genre fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}