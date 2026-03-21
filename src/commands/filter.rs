use poise::CreateReply;
use crate::{
    api::anilist::fetch_filtered_media,
    models::bot_data::{Context, Error, MediaType, MediaFormat, MediaStatus, MediaSort},
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

/// Advanced search/filter for media.
#[poise::command(slash_command, prefix_command)]
pub async fn filter(
    ctx: Context<'_>,
    #[description = "Media type (ANIME or MANGA)"] media_type: Option<MediaType>,
    #[description = "Media format (TV, Movie, etc.)"] format: Option<MediaFormat>,
    #[description = "Media status (Finished, Releasing, etc.)"] status: Option<MediaStatus>,
    #[description = "Genre to filter by"] #[autocomplete = "autocomplete_genre"] genre: Option<String>,
    #[description = "Release year"] year: Option<i32>,
    #[description = "Sort order"] sort: Option<MediaSort>,
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

    let media_type_str = media_type.map(|t| t.as_str());
    let format_str = format.map(|f| f.as_str());
    let format_vec = format_str.as_ref().map(|s| vec![*s]);
    let status_str = status.map(|s| s.as_str());
    
    let genres_vec = genre.as_deref().map(|g| vec![g]);
    let sort_str = sort.map(|s| s.as_str());
    let sort_vec = sort_str.as_ref().map(|s| vec![*s]);

    match fetch_filtered_media(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        media_type_str,
        format_vec,
        status_str,
        None, // country
        genres_vec,
        year,
        sort_vec
    ).await {
        Ok(media) => {
            ctx.send(CreateReply::default().embed(media_list_embed(&media, "Search Results", prefs.title_language, accent_color))).await?;
        }
        Err(e) => {
            tracing::warn!("Filter fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
