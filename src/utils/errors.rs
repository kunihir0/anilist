use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

use crate::api::anilist::{AniListError, AniListErrorKind};
use crate::models::bot_data::{Context, Error};

// ─── Colour palette ───────────────────────────────────────────────────────────
const RED:    u32 = 0xe74c3c;
const ORANGE: u32 = 0xe67e22;
const YELLOW: u32 = 0xf1c40f;
const GREY:   u32 = 0x95a5a6;

// ─── Categorised error embed ──────────────────────────────────────────────────

/// Build a rich error embed whose title, colour, and advice vary by error kind.
///
/// Downcasts `err` to `AniListError` for structured handling; falls back to a
/// generic embed for any other error type.
pub fn error_embed_from(err: &Error) -> CreateEmbed {
    if let Some(al) = err.downcast_ref::<AniListError>() {
        return anilist_error_embed(al);
    }

    // Generic fallback
    CreateEmbed::new()
        .title("Something went wrong")
        .description(format!("```\n{err}\n```"))
        .colour(poise::serenity_prelude::Colour::new(RED))
        .footer(poise::serenity_prelude::CreateEmbedFooter::new(
            "If this keeps happening, the bot may be misconfigured.",
        ))
}

fn anilist_error_embed(err: &AniListError) -> CreateEmbed {
    let (colour, title, advice) = match err.kind {
        AniListErrorKind::NotFound => (
            GREY,
            "Not Found",
            "Double-check the spelling. AniList search is exact — try the Romaji or English title.",
        ),
        AniListErrorKind::RateLimit => (
            YELLOW,
            "Rate Limited",
            "The bot has sent too many requests to AniList. Wait a few seconds and try again.",
        ),
        AniListErrorKind::Network => (
            ORANGE,
            "Network Error",
            "Could not reach the AniList API. This is usually temporary — try again shortly.",
        ),
        AniListErrorKind::Decode => (
            ORANGE,
            "Unexpected Response",
            "AniList returned data in an unexpected format. The API may have changed.",
        ),
        AniListErrorKind::ApiError => (
            RED,
            "AniList API Error",
            "AniList returned an error. See the detail below.",
        ),
    };

    let mut embed = CreateEmbed::new()
        .title(format!("❌  {title}"))
        .colour(poise::serenity_prelude::Colour::new(colour))
        .field("Detail", format!("```\n{}\n```", err.message), false)
        .field("Advice", advice, false);

    if let Some(code) = err.status {
        embed = embed.footer(poise::serenity_prelude::CreateEmbedFooter::new(
            format!("HTTP status {code}"),
        ));
    }

    embed
}

// ─── Not-found helper ─────────────────────────────────────────────────────────

/// Convenience embed for empty result sets (not an API error, just no match).
pub fn not_found_embed(entity: &str, query: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("❌  {entity} Not Found"))
        .description(format!(
            "No results for **{query}**.\nTry a different spelling or use the Romaji title."
        ))
        .colour(poise::serenity_prelude::Colour::new(GREY))
}

// ─── Command-level error reply ────────────────────────────────────────────────

/// Send an ephemeral error reply built from any `Error`.
/// Call this at the end of every `Err` arm in command handlers.
pub async fn reply_error(ctx: Context<'_>, err: &Error) -> Result<(), Error> {
    ctx.send(
        CreateReply::default()
            .embed(error_embed_from(err))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}