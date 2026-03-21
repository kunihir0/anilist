use chrono::{Datelike, Utc};
use poise::CreateReply;
use crate::{
    api::anilist::fetch_upcoming,
    models::bot_data::{Context, Error},
    utils::{
        embeds::upcoming_page_embed,
        errors::{not_found_embed, reply_error},
        pagination::paginate,
    },
};

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

    // Default to current season/year if not provided
    let now = Utc::now();
    let year_val = year.unwrap_or(now.year());
    let season_str = season.unwrap_or_else(|| {
        match now.month() {
            1 | 2 | 3 => "WINTER",
            4 | 5 | 6 => "SPRING",
            7 | 8 | 9 => "SUMMER",
            _ => "FALL",
        }
        .to_string()
    });

    match fetch_upcoming(&data.http_client, &data.cache, &data.rate_limiter, &season_str, year_val).await {
        Ok(results) if results.is_empty() => {
            ctx.send(CreateReply::default().embed(not_found_embed("Upcoming", &format!("{season_str} {year_val}")))).await?;
        }
        Ok(results) => {
            let chunks: Vec<_> = results.chunks(5).collect();
            let total_pages = chunks.len();
            let pages: Vec<_> = chunks
                .iter()
                .enumerate()
                .map(|(i, chunk)| {
                    upcoming_page_embed(chunk, &season_str, year_val, i + 1, total_pages, prefs.title_language.clone())
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
