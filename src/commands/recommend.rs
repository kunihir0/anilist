use crate::{
    api::anilist::fetch_watchlist,
    models::bot_data::{Context, Error},
    utils::embeds::media_list_embed,
};
use poise::CreateReply;
use std::collections::HashSet;

/// Comparison-based recommendations.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("compare"),
    category = "Social"
)]
pub async fn recommend(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Suggest what one user has seen that another hasn't.
#[poise::command(slash_command, prefix_command)]
pub async fn compare(
    ctx: Context<'_>,
    #[description = "The user who has seen things"] user1: String,
    #[description = "The user to suggest to"] user2: String,
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

    // Fetch both watchlists (Anime by default for now)
    let list1 = fetch_watchlist(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &user1,
        "ANIME",
    )
    .await?;
    let list2 = fetch_watchlist(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &user2,
        "ANIME",
    )
    .await?;

    // Find titles in user1's "Completed" list that aren't in user2's list at all
    let user2_media_ids: HashSet<u64> = list2
        .lists
        .iter()
        .flat_map(|l| l.entries.iter().map(|e| e.media.id))
        .collect();

    let mut recommendations = Vec::new();
    for list in list1.lists {
        if list.name == "Completed" {
            for entry in list.entries {
                if !user2_media_ids.contains(&entry.media.id) {
                    // We need full Media objects for media_list_embed,
                    // but we only have MediaListNode. Let's build a simple Media shim or
                    // fetch full info. For now, let's build a minimal Media object.
                    // Ideally we'd update media_list_embed to handle minimal info.
                    // Or we just display a text list.
                    recommendations.push(entry.media);
                }
            }
        }
    }

    if recommendations.is_empty() {
        ctx.say(format!(
            "Found no completed anime from **{}** that **{}** hasn't started.",
            user1, user2
        ))
        .await?;
        return Ok(());
    }

    // Convert recommendations to minimal Media objects for the embed
    use crate::models::responses::Media;
    let media_recs: Vec<Media> = recommendations
        .into_iter()
        .take(10)
        .map(|r| Media {
            id: r.id,
            title: r.title,
            episodes: None,
            season: None,
            season_year: None,
            chapters: None,
            volumes: None,
            format: None,
            status: None,
            average_score: None,
            genres: vec![],
            description: None,
            cover_image: None,
            site_url: r.site_url,
            start_date: None,
            next_airing_episode: None,
            relations: None,
            characters: None,
            recommendations: None,
        })
        .collect();

    let title = format!("Suggestions for {} from {}'s list", user2, user1);
    ctx.send(CreateReply::default().embed(media_list_embed(
        &media_recs,
        &title,
        prefs.title_language,
        accent_color,
    )))
    .await?;

    Ok(())
}
