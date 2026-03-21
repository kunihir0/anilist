mod anime;
mod airing;
mod character;
mod compare;
mod favourites;
mod genre;
mod manga;
mod profile;
mod random;
mod recommendations;
mod staff;
mod studio;
mod trending;
mod upcoming;

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
    ]
}