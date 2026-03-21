mod anime;
mod airing;
mod character;
mod compare;
mod favourites;
mod filter;
mod genre;
mod help;
mod manga;
mod prefs;
mod profile;
mod quiz;
mod random;
mod recommendations;
mod relations;
pub mod schedule;
pub mod settings;
mod staff;
mod studio;
mod trending;
mod upcoming;
mod watch;
mod watchlist;

use crate::models::bot_data::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        anime::anime(),
        manga::manga(),
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
    ]
}
