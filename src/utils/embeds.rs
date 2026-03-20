use poise::serenity_prelude::{self as serenity, CreateEmbed};

use crate::models::responses::{Media, AniListUser};

// AniList brand colour (#02a9ff)
const ANILIST_BLUE: u32 = 0x02a9ff;
// Mild red for error embeds
const ERROR_RED: u32 = 0xe74c3c;

// ─── Media embed (shared by /anime and /manga) ───────────────────────────────

/// Build a rich Discord embed for any `Media` entry (anime or manga).
///
/// The `media_type` label ("Anime" / "Manga") is used in the embed footer.
pub fn media_embed(media: &Media, media_type: &str) -> CreateEmbed {
    // Clean up the AniList description: strip residual HTML tags and truncate
    // to 300 chars so the embed doesn't overflow.
    let description = media
        .description
        .as_deref()
        .map(clean_html)
        .map(|d| truncate(&d, 300))
        .unwrap_or_else(|| "No description available.".to_string());

    let title = media.title.preferred();
    let genres = if media.genres.is_empty() {
        "N/A".to_string()
    } else {
        media.genres.join(", ")
    };

    let score = media
        .average_score
        .map(|s| format!("{s}/100"))
        .unwrap_or_else(|| "N/A".to_string());

    let status = media.status.as_deref().unwrap_or("Unknown");
    let format = media.format.as_deref().unwrap_or("Unknown");

    let mut embed = CreateEmbed::new()
        .title(title)
        .url(&media.site_url)
        .description(description)
        .color(serenity::Colour::new(ANILIST_BLUE))
        .footer(serenity::CreateEmbedFooter::new(format!(
            "{media_type} • AniList ID {}", media.id
        )))
        .field("Format", format, true)
        .field("Status", status, true)
        .field("Score", score, true)
        .field("Genres", genres, false)
        .field("Start Date", media.start_date.display(), true);

    // Anime-specific: episode count + season
    if let Some(eps) = media.episodes {
        embed = embed.field("Episodes", eps.to_string(), true);
    }
    if let (Some(season), Some(year)) = (&media.season, media.season_year) {
        embed = embed.field("Season", format!("{season} {year}"), true);
    }

    // Manga-specific: chapters + volumes
    if let Some(ch) = media.chapters {
        embed = embed.field("Chapters", ch.to_string(), true);
    }
    if let Some(vol) = media.volumes {
        embed = embed.field("Volumes", vol.to_string(), true);
    }

    // Thumbnail from cover image
    if let Some(img_url) = &media.cover_image.large {
        embed = embed.thumbnail(img_url);
    }

    embed
}

// ─── User profile embed ──────────────────────────────────────────────────────

pub fn user_embed(user: &AniListUser) -> CreateEmbed {
    let about = user
        .about
        .as_deref()
        .map(clean_html)
        .map(|a| truncate(&a, 200))
        .unwrap_or_else(|| "No bio set.".to_string());

    let anime = &user.statistics.anime;
    let manga = &user.statistics.manga;

    // Convert total minutes to a friendly "Xd Yh" string
    let days = anime.minutes_watched / 1440;
    let hours = (anime.minutes_watched % 1440) / 60;
    let watch_time = format!("{days}d {hours}h");

    let mut embed = CreateEmbed::new()
        .title(&user.name)
        .url(&user.site_url)
        .description(about)
        .color(serenity::Colour::new(ANILIST_BLUE))
        .footer(serenity::CreateEmbedFooter::new(format!(
            "AniList User ID {}", user.id
        )))
        // Anime stats
        .field("Anime Watched", anime.count.to_string(), true)
        .field("Episodes Watched", anime.episodes_watched.to_string(), true)
        .field("Watch Time", watch_time, true)
        .field("Anime Mean Score", format!("{:.1}/100", anime.mean_score), true)
        // Manga stats
        .field("Manga Read", manga.count.to_string(), true)
        .field("Chapters Read", manga.chapters_read.to_string(), true)
        .field("Manga Mean Score", format!("{:.1}/100", manga.mean_score), true);

    if let Some(avatar_url) = &user.avatar.large {
        embed = embed.thumbnail(avatar_url);
    }

    embed
}

// ─── Error embed ─────────────────────────────────────────────────────────────

/// A simple red embed used by the centralised error handler.
pub fn error_embed(title: &str, body: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("❌  {title}"))
        .description(body)
        .color(serenity::Colour::new(ERROR_RED))
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/// Very lightweight HTML tag stripper.  AniList descriptions occasionally
/// contain `<br>`, `<i>`, `<b>`, etc., which look terrible as plain text.
fn clean_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    // Collapse excessive whitespace produced by stripping tags
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Truncate a string to `max` chars, appending "…" if it was cut.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}