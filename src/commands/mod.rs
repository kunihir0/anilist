mod airing;
mod character;
mod compare;
mod favourites;
mod filter;
mod genre;
mod help;
mod leaderboard;
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
mod studio;
mod trending;
mod upcoming;
mod watch;
mod watchlist;

use crate::models::bot_data::{Data, Error};

macro_rules! make_search_command {
    ($func_name:ident, $fetch_fn:path, $media_type:expr, $description:expr) => {
        #[poise::command(slash_command, prefix_command, user_cooldown = 5, category = "Search")]
        #[doc = $description]
        pub async fn $func_name(
            ctx: crate::models::bot_data::Context<'_>,
            #[description = "Title to search for"] title: String,
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

            match $fetch_fn(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
                Ok(results) if results.is_empty() => {
                    ctx.say(format!(
                        "No {} found for `{title}`.",
                        stringify!($func_name)
                    ))
                    .await?;
                }
                Ok(results) => {
                    let pages: Vec<_> = results
                        .iter()
                        .map(|m| {
                            crate::utils::embeds::media_embed(
                                m,
                                $media_type,
                                prefs.title_language.clone(),
                                accent_color,
                            )
                        })
                        .collect();
                    crate::utils::pagination::paginate(ctx, pages).await?;
                }
                Err(e) => {
                    tracing::warn!("{} fetch failed for {title:?}: {e}", $media_type);
                    crate::utils::errors::reply_error(ctx, &e).await?;
                }
            }

            Ok(())
        }
    };
}

make_search_command!(
    anime,
    crate::api::anilist::fetch_anime,
    "Anime",
    "Search AniList for an anime by title."
);
make_search_command!(
    manga,
    crate::api::anilist::fetch_manga,
    "Manga",
    "Search AniList for a manga by title."
);

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
    ]
}
