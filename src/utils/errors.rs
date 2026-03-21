use crate::api::anilist::AniListError;
use crate::models::bot_data::{BotError, Context, Error};
use poise::serenity_prelude::{self as serenity, CreateEmbed};

const RED: u32 = 0xe74c3c;
const YELLOW: u32 = 0xf1c40f;
const ORANGE: u32 = 0xe67e22;

/// Create a generic error embed.
pub fn error_embed(msg: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title("Error")
        .description(msg)
        .colour(serenity::Colour::new(RED))
}

/// Create an embed specifically for AniList API errors.
pub fn anilist_error_embed(err: &AniListError) -> CreateEmbed {
    let (colour, title, advice) = match err {
        AniListError::NotFound { .. } => (
            YELLOW,
            "Not Found",
            "Try adjusting your search terms or checking the spelling.",
        ),
        AniListError::RateLimit => (
            ORANGE,
            "Rate Limited",
            "AniList allows 90 requests per minute. Please wait a moment before trying again.",
        ),
        AniListError::Api { .. } => (
            RED,
            "AniList API Error",
            "The AniList API returned an error. This might be a temporary issue.",
        ),
        AniListError::Network(_) => (
            RED,
            "Network Error",
            "Failed to connect to AniList. Check your internet connection or try again later.",
        ),
        AniListError::Decode(_) => (
            RED,
            "Decoding Error",
            "Received an unexpected response format from AniList.",
        ),
    };

    let mut embed = CreateEmbed::new()
        .title(title)
        .colour(serenity::Colour::new(colour))
        .footer(serenity::CreateEmbedFooter::new(advice));

    match err {
        AniListError::Api { message, status } => {
            embed = embed.field("Message", format!("```\n{}\n```", message), false);
            if let Some(code) = status {
                embed = embed.field("Status Code", code.to_string(), true);
            }
        }
        AniListError::NotFound { message } => {
            embed = embed.field("Message", format!("```\n{}\n```", message), false);
        }
        AniListError::Network(msg) | AniListError::Decode(msg) => {
            embed = embed.field("Details", format!("```\n{}\n```", msg), false);
        }
        AniListError::RateLimit => {}
    }

    embed
}

/// Standardized error reply function for commands.
pub async fn reply_error(ctx: Context<'_>, err: &Error) -> Result<(), Error> {
    let embed = match err {
        BotError::Api(al_err) => anilist_error_embed(al_err),
        _ => error_embed(&err.to_string()),
    };

    let reply = poise::CreateReply::default().embed(embed).ephemeral(true);
    let _ = ctx.send(reply).await;
    Ok(())
}

/// Specific embed for "No results found" (not a hard error).
pub fn not_found_embed(entity: &str, query: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("No {} Found", entity))
        .description(format!("Could not find any results for `{}`.", query))
        .colour(serenity::Colour::new(YELLOW))
}
