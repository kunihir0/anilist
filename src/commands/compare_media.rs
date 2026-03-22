use crate::{
    api::anilist::{fetch_anime, fetch_manga},
    commands::autocomplete_anime,
    models::bot_data::{Context, Error, MediaType},
    utils::{embeds::media_compare_embed, errors::reply_error},
};
use poise::CreateReply;

/// Compare two media items (anime/manga) side by side.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn compare_media(
    ctx: Context<'_>,
    #[description = "First title"]
    #[autocomplete = "autocomplete_anime"]
    title1: String,
    #[description = "Second title"]
    #[autocomplete = "autocomplete_anime"]
    title2: String,
    #[description = "First media type (default: ANIME)"] type1: Option<MediaType>,
    #[description = "Second media type (default: ANIME)"] type2: Option<MediaType>,
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

    let kind1 = type1.unwrap_or(MediaType::Anime);
    let kind2 = type2.unwrap_or(MediaType::Anime);

    // Prepare async fetches based on types
    let f1 = async {
        match kind1 {
            MediaType::Anime => {
                fetch_anime(&data.http_client, &data.cache, &data.rate_limiter, &title1).await
            }
            MediaType::Manga => {
                fetch_manga(&data.http_client, &data.cache, &data.rate_limiter, &title1).await
            }
        }
    };
    let f2 = async {
        match kind2 {
            MediaType::Anime => {
                fetch_anime(&data.http_client, &data.cache, &data.rate_limiter, &title2).await
            }
            MediaType::Manga => {
                fetch_manga(&data.http_client, &data.cache, &data.rate_limiter, &title2).await
            }
        }
    };

    match tokio::try_join!(f1, f2) {
        Ok((res1, res2)) => {
            let m1 = res1.into_iter().next();
            let m2 = res2.into_iter().next();

            match (m1, m2) {
                (Some(m1), Some(m2)) => {
                    ctx.send(CreateReply::default().embed(media_compare_embed(
                        &m1,
                        &m2,
                        prefs.title_language,
                        accent_color,
                    )))
                    .await?;
                }
                (None, _) => {
                    ctx.say(format!("Could not find `{}`.", title1)).await?;
                }
                (_, None) => {
                    ctx.say(format!("Could not find `{}`.", title2)).await?;
                }
            }
        }
        Err(e) => {
            tracing::warn!("Media comparison fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
