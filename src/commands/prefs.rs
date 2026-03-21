use crate::models::bot_data::{Context, Error};
use crate::store::TitleLanguage;

/// Manage your personal preferences.
#[poise::command(slash_command, prefix_command, subcommands("title_language"))]
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
