use poise::CreateReply;

use crate::{
    api::anilist::fetch_anime,
    models::bot_data::{Context, Error},
    utils::embeds::{error_embed, media_embed},
};

/// Search AniList for an anime by title and display a rich info card.
#[poise::command(slash_command, prefix_command)]
pub async fn anime(
    ctx: Context<'_>,
    #[description = "Anime title to search for"] title: String,
) -> Result<(), Error> {
    // Defer the response so Discord doesn't time out if the API is slow.
    ctx.defer().await?;

    match fetch_anime(&ctx.data().http_client, &title).await {
        Ok(media) => {
            let embed = media_embed(&media, "Anime");
            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            tracing::warn!("AniList anime fetch failed for {:?}: {e}", title);
            let embed = error_embed(
                "Anime Not Found",
                &format!("Could not find an anime matching **{title}**.\nDouble-check the spelling or try a different title."),
            );
            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
