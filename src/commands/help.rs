use crate::models::bot_data::{Context, Error};
use poise::serenity_prelude as serenity;
use std::collections::BTreeMap;

/// Show a list of all available commands.
#[poise::command(slash_command, prefix_command, category = "Utility")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help for"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    if let Some(cmd_name) = &command {
        // Show help for a specific command
        let commands = &ctx.framework().options().commands;
        if let Some(cmd) = commands.iter().find(|c| c.name == *cmd_name) {
            let desc = cmd
                .description
                .as_deref()
                .unwrap_or("No description available.");
            let mut embed = serenity::CreateEmbed::new()
                .title(format!("/{}", cmd.name))
                .description(desc)
                .colour(0x2ECC71);

            if !cmd.subcommands.is_empty() {
                let subs: String = cmd
                    .subcommands
                    .iter()
                    .map(|s| {
                        format!(
                            "`{}` — {}",
                            s.name,
                            s.description.as_deref().unwrap_or("No description")
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                embed = embed.field("Subcommands", subs, false);
            }

            ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
                .await?;
        } else {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("Command `{}` not found.", cmd_name))
                    .ephemeral(true),
            )
            .await?;
        }
    } else {
        // Group commands by category
        let commands = &ctx.framework().options().commands;
        let mut categories: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for cmd in commands {
            let cat = cmd.category.as_deref().unwrap_or("Other");
            categories.entry(cat).or_default().push(&cmd.name);
        }

        let mut embed = serenity::CreateEmbed::new()
            .title("📖 Command List")
            .description("Use `/help <command>` for details on a specific command.")
            .colour(0x3498DB)
            .timestamp(serenity::Timestamp::now());

        for (cat, cmds) in &categories {
            let list = cmds
                .iter()
                .map(|c| format!("`/{c}`"))
                .collect::<Vec<_>>()
                .join("  ");
            embed = embed.field(*cat, list, false);
        }

        ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
            .await?;
    }
    Ok(())
}
