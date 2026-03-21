use poise::CreateReply;
use crate::{
    api::anilist::fetch_random,
    models::bot_data::{Context, Error},
    utils::{
        embeds::media_embed,
        errors::reply_error,
    },
};

/// Discord slash-command choice: Anime or Manga.
#[derive(Debug, poise::ChoiceParameter)]
pub enum MediaType {
    #[name = "Anime"]
    Anime,
    #[name = "Manga"]
    Manga,
}

impl MediaType {
    fn as_api_str(&self) -> &'static str {
        match self {
            MediaType::Anime => "ANIME",
            MediaType::Manga => "MANGA",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            MediaType::Anime => "Anime",
            MediaType::Manga => "Manga",
        }
    }
}

/// Get a random well-rated anime or manga from AniList.
#[poise::command(slash_command, prefix_command)]
pub async fn random(
    ctx: Context<'_>,
    #[description = "Anime or Manga"] media_type: Option<MediaType>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    let kind = media_type.unwrap_or(MediaType::Anime);

    match fetch_random(&data.http_client, &data.rate_limiter, kind.as_api_str()).await {
        Ok(media) => {
            ctx.send(CreateReply::default().embed(media_embed(&media, kind.label()))).await?;
        }
        Err(e) => {
            tracing::warn!("Random fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}