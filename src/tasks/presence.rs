use std::time::Duration;

use poise::serenity_prelude::{self as serenity, ActivityData, OnlineStatus};
use tokio::time;

/// All 10 rotating statuses.
///
/// Each entry is a `(kind, text)` pair where `kind` maps to the
/// `ActivityData` constructor we call.  Cycle time: 30 seconds per status.
const STATUSES: &[(&str, &str)] = &[
    ("watching", "anime on AniList"),
    ("listening", "lo-fi while reading manga"),
    ("playing", "with Poise & Serenity"),
    ("watching", "your AniList stats"),
    ("competing", "Best Anime of the Season"),
    ("listening", "the Attack on Titan OST"),
    ("watching", "One Piece (still going)"),
    ("playing", "/anime to search titles"),
    ("listening", "voice lines from Genshin"),
    ("watching", "the seasonal chart"),
];

const INTERVAL_SECS: u64 = 90;

/// Spawns a background task that cycles through [`STATUSES`] indefinitely,
/// updating the bot's Discord presence every [`INTERVAL_SECS`] seconds.
///
/// Call this once inside the Poise `setup` callback after the `http` context
/// is available.
pub fn spawn(ctx: serenity::Context) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(INTERVAL_SECS));
        let mut index: usize = 0;

        loop {
            interval.tick().await;

            let (kind, text) = STATUSES[index % STATUSES.len()];
            let activity = match kind {
                "watching" => ActivityData::watching(text),
                "listening" => ActivityData::listening(text),
                "competing" => ActivityData::competing(text),
                _ => ActivityData::playing(text), // "playing" + fallback
            };

            ctx.set_presence(Some(activity), OnlineStatus::Online);

            index = index.wrapping_add(1);
        }
    });
}
