use crate::models::bot_data::{Context, Error};
use crate::store::TitleLanguage;

/// Manage your personal preferences.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("title_language", "compact_mode"),
    category = "Utility"
)]
pub async fn prefs(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set your preferred title language for embeds.
#[poise::command(slash_command, prefix_command)]
pub async fn title_language(
    ctx: Context<'_>,
    #[description = "Preferred language"] language: TitleLanguage,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    ctx.data()
        .store
        .set_title_language(user_id, language.clone())
        .await?;

    ctx.say(format!(
        "Title language preference updated to **{:?}**.",
        language
    ))
    .await?;
    Ok(())
}

/// Toggle compact embed mode.
#[poise::command(slash_command, prefix_command)]
pub async fn compact_mode(
    ctx: Context<'_>,
    #[description = "Enable compact mode?"] enable: Option<bool>,
) -> Result<(), Error> {
    let user_id = ctx.author().id.get();
    let old_prefs = ctx.data().store.get_user_prefs(user_id).await;

    let new_mode = enable.unwrap_or(!old_prefs.compact_mode);

    ctx.data().store.set_compact_mode(user_id, new_mode).await?;

    let state_str = if new_mode { "enabled" } else { "disabled" };
    ctx.say(format!("Compact mode has been **{}**.", state_str))
        .await?;
    Ok(())
}
