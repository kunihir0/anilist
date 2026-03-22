mod airing;
mod character;
mod compare;
mod compare_media;
mod favourites;
mod filter;
mod genre;
mod help;
mod leaderboard;
mod notify;
mod ping;
mod prefs;
mod profile;
mod quiz;
mod random;
mod recommend;
mod recommendations;
mod relations;
pub mod schedule;
mod serverlist;
pub mod settings;
mod staff;
mod stats;
mod studio;
mod tag;
mod trending;
mod upcoming;
mod watch;
mod watchlist;

use crate::models::bot_data::{Data, Error};

// ─── Autocomplete handlers ───────────────────────────────────────────────────

pub async fn autocomplete_anime(
    ctx: crate::models::bot_data::Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    autocomplete_media(ctx, partial, "ANIME").await
}

pub async fn autocomplete_manga(
    ctx: crate::models::bot_data::Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    autocomplete_media(ctx, partial, "MANGA").await
}

async fn autocomplete_media(
    ctx: crate::models::bot_data::Context<'_>,
    partial: &str,
    media_type: &str,
) -> std::vec::IntoIter<poise::serenity_prelude::AutocompleteChoice> {
    if partial.len() < 2 {
        return Vec::new().into_iter();
    }
    let data = ctx.data();
    match crate::api::anilist::fetch_media_autocomplete(
        &data.http_client,
        &data.rate_limiter,
        partial,
        media_type,
    )
    .await
    {
        Ok(items) => items
            .into_iter()
            .map(|m| {
                let label = match m.format.as_deref() {
                    Some(fmt) => format!("{} ({})", m.title.preferred(), fmt),
                    None => m.title.preferred().to_string(),
                };
                poise::serenity_prelude::AutocompleteChoice::new(
                    label,
                    m.title.preferred().to_string(),
                )
            })
            .collect::<Vec<_>>()
            .into_iter(),
        Err(_) => Vec::new().into_iter(),
    }
}

// ─── Search commands ─────────────────────────────────────────────────────────

/// Search AniList for an anime by title.
#[poise::command(slash_command, prefix_command, user_cooldown = 5, category = "Search")]
pub async fn anime(
    ctx: crate::models::bot_data::Context<'_>,
    #[description = "Title to search for"]
    #[autocomplete = "autocomplete_anime"]
    title: String,
) -> Result<(), crate::models::bot_data::Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;
    let guild_id = ctx.guild_id().map(|id| id.get());
    let accent_color = if let Some(gid) = guild_id {
        data.store.get_settings(gid).await.accent_color
    } else {
        None
    };

    match crate::api::anilist::fetch_anime(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &title,
    )
    .await
    {
        Ok(results) if results.is_empty() => {
            ctx.say(format!("No Anime found for `{title}`.")).await?;
        }
        Ok(results) => {
            let pages: Vec<_> = results
                .into_iter()
                .map(|m| {
                    let embed = crate::utils::embeds::media_embed(
                        &m,
                        "Anime",
                        prefs.title_language.clone(),
                        accent_color,
                        prefs.compact_mode,
                    );
                    (embed, m)
                })
                .collect();
            crate::utils::pagination::paginate_media(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Anime fetch failed for {title:?}: {e}");
            crate::utils::errors::reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}

/// Search AniList for a manga by title.
#[poise::command(slash_command, prefix_command, user_cooldown = 5, category = "Search")]
pub async fn manga(
    ctx: crate::models::bot_data::Context<'_>,
    #[description = "Title to search for"]
    #[autocomplete = "autocomplete_manga"]
    title: String,
) -> Result<(), crate::models::bot_data::Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;
    let guild_id = ctx.guild_id().map(|id| id.get());
    let accent_color = if let Some(gid) = guild_id {
        data.store.get_settings(gid).await.accent_color
    } else {
        None
    };

    match crate::api::anilist::fetch_manga(
        &data.http_client,
        &data.cache,
        &data.rate_limiter,
        &title,
    )
    .await
    {
        Ok(results) if results.is_empty() => {
            ctx.say(format!("No Manga found for `{title}`.")).await?;
        }
        Ok(results) => {
            let pages: Vec<_> = results
                .into_iter()
                .map(|m| {
                    let embed = crate::utils::embeds::media_embed(
                        &m,
                        "Manga",
                        prefs.title_language.clone(),
                        accent_color,
                        prefs.compact_mode,
                    );
                    (embed, m)
                })
                .collect();
            crate::utils::pagination::paginate_media(ctx, pages).await?;
        }
        Err(e) => {
            tracing::warn!("Manga fetch failed for {title:?}: {e}");
            crate::utils::errors::reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        anime(),
        manga(),
        profile::profile(),
        character::character(),
        studio::studio(),
        staff::staff(),
        recommendations::recommendations(),
        trending::trending(),
        genre::genre(),
        favourites::favourites(),
        upcoming::upcoming(),
        airing::airing(),
        random::random(),
        compare::compare(),
        schedule::schedule(),
        settings::settings(),
        watchlist::watchlist(),
        relations::relations(),
        filter::filter(),
        prefs::prefs(),
        help::help(),
        quiz::quiz(),
        watch::watch(),
        serverlist::serverlist(),
        leaderboard::leaderboard(),
        recommend::recommend(),
        ping::ping(),
        tag::tag(),
        compare_media::compare_media(),
        notify::notify(),
        stats::stats(),
    ]
}
