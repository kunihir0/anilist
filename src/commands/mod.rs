mod anime;
mod airing;
mod character;
mod compare;
mod manga;
mod profile;
mod random;
mod studio;
mod upcoming;

use crate::models::bot_data::{Data, Error};

pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        anime::anime(),
        manga::manga(),
        profile::profile(),
        character::character(),
        studio::studio(),
        upcoming::upcoming(),
        airing::airing(),
        random::random(),
        compare::compare(),
    ]
}