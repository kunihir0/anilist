use poise::CreateReply;

use crate::{
    api::anilist::fetch_manga,
    models::bot_data::{Context, Error},
    utils::embeds::{error_embed, media_embed},
};

/// Search AniList for a manga by title and display a rich info card.
#[poise::command(slash_command, prefix_command)]
pub async fn manga(
    ctx: Context<'_>,
    #[description = "Manga title to search for"] title: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    match fetch_manga(&ctx.data().http_client, &title).await {
        Ok(media) => {
            let embed = media_embed(&media, "Manga");
            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            tracing::warn!("AniList manga fetch failed for {:?}: {e}", title);
            let embed = error_embed(
                "Manga Not Found",
                &format!("Could not find a manga matching **{title}**.\nDouble-check the spelling or try a different title."),
            );
            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
