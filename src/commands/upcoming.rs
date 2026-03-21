use crate::{
    api::anilist::fetch_upcoming,
    models::bot_data::{Context, Error},
    utils::{
        embeds::upcoming_page_embed,
        errors::{not_found_embed, reply_error},
        pagination::paginate,
    },
};
use chrono::{Datelike, Utc};
use poise::CreateReply;

/// Show upcoming anime for a given season and year.
#[poise::command(slash_command, prefix_command)]
pub async fn upcoming(
    ctx: Context<'_>,
    #[description = "Season (WINTER, SPRING, SUMMER, FALL)"] season: Option<String>,
    #[description = "Year (e.g. 2024)"] year: Option<i32>,
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

    // Default to current season/year if not provided
    let now = Utc::now();
    let year_val = year.unwrap_or(now.year());
    let season_str = season.unwrap_or_else(|| {
        match now.month() {
            1..=3 => "WINTER",
            4..=6 => "SPRING",
            7..=9 => "SUMMER",
            _ => "FALL",
        }
        .to_string()
    });

    match fetch_upcoming(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &season_str,
        year_val,
    )
    .await
    {
        Ok(results) if results.is_empty() => {
            ctx.send(CreateReply::default().embed(not_found_embed(
                "Upcoming",
                &format!("{season_str} {year_val}"),
            )))
            .await?;
        }
        Ok(results) => {
            let chunks: Vec<_> = results.chunks(5).collect();
            let total_pages = chunks.len();
            let pages: Vec<_> = chunks
                .iter()
                .enumerate()
                .map(|(i, chunk)| {
                    upcoming_page_embed(
                        chunk,
                        &season_str,
                        year_val,
                        i + 1,
                        total_pages,
                        prefs.title_language.clone(),
                        accent_color,
                    )
                })
                .collect();
            paginate(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Upcoming fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
