use crate::{
    api::anilist::fetch_airing,
    models::bot_data::{Context, Error},
    utils::{embeds::airing_page_embed, errors::reply_error, pagination::paginate},
};
use chrono::{Datelike, TimeZone, Utc, Weekday};

fn parse_day(day: &str) -> Option<Weekday> {
    match day.to_lowercase().as_str() {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        "today" => Some(Utc::now().weekday()),
        "tomorrow" => Some(Utc::now().weekday().succ()),
        _ => None,
    }
}

/// Show currently airing anime with episode countdowns.
#[poise::command(
    slash_command,
    prefix_command,
    user_cooldown = 5,
    category = "Discovery"
)]
pub async fn airing(
    ctx: Context<'_>,
    #[description = "Filter by day (Monday, Tuesday, etc., or 'today')"] day: Option<String>,
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

    match fetch_airing(&data.http_client, &data.cache, &data.rate_limiter).await {
        Ok(mut shows) => {
            let target_day = day.as_deref().and_then(parse_day);

            if let Some(target) = target_day {
                shows.retain(|show| {
                    if let Some(ep) = &show.next_airing_episode {
                        // AniList provides airing_at in seconds UTC
                        if let chrono::LocalResult::Single(dt) = Utc.timestamp_opt(ep.airing_at, 0)
                        {
                            return dt.weekday() == target;
                        }
                    }
                    false
                });
            }

            if shows.is_empty() {
                if let Some(ref d) = day {
                    ctx.say(format!("No currently airing anime found for `{}`.", d))
                        .await?;
                } else {
                    ctx.say("No currently airing anime found.").await?;
                }
                return Ok(());
            }

            let day_filter = day.as_deref().map(|s| {
                let mut c = s.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            });

            // Group shows into chunks of 5 for pagination
            let chunks: Vec<_> = shows.chunks(5).collect();
            let total_pages = chunks.len();
            let pages: Vec<_> = chunks
                .iter()
                .enumerate()
                .map(|(i, chunk)| {
                    airing_page_embed(
                        chunk,
                        i + 1,
                        total_pages,
                        day_filter.as_deref(),
                        prefs.title_language.clone(),
                        accent_color,
                    )
                })
                .collect();

            paginate(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Airing fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
