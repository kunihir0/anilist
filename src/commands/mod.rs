mod anime;
mod manga;
mod profile;

use crate::models::bot_data::{Data, Error};

/// Returns all registered commands.  Pass this directly to
/// `FrameworkOptions { commands: commands::all(), … }` in main.rs.
pub fn all() -> Vec<poise::Command<Data, Error>> {
    vec![
        anime::anime(),
        manga::manga(),
        profile::profile(),
    ]
}
