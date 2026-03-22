use crate::models::bot_data::{Context, Error};

/// Show a list of all available commands.
#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help for"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    let configuration = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Type /help <command> for more info on a specific command.",
        ephemeral: true,
        show_subcommands: true,
        include_description: true,
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}
