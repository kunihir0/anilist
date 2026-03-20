use poise::CreateReply;

use crate::{
    api::anilist::fetch_user,
    models::bot_data::{Context, Error},
    utils::embeds::{error_embed, user_embed},
};

/// Look up a public AniList user profile and display their stats.
#[poise::command(slash_command, prefix_command)]
pub async fn profile(
    ctx: Context<'_>,
    #[description = "AniList username to look up"] username: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    match fetch_user(&ctx.data().http_client, &username).await {
        Ok(user) => {
            let embed = user_embed(&user);
            ctx.send(CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            tracing::warn!("AniList user fetch failed for {:?}: {e}", username);
            let embed = error_embed(
                "User Not Found",
                &format!("Could not find an AniList profile for **{username}**.\nMake sure the username is spelled correctly and the profile is public."),
            );
            ctx.send(CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        }
    }

    Ok(())
}
