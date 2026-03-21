use crate::models::bot_data::{Context, Error};
use crate::utils::permissions::check_admin_or_mod;
use poise::serenity_prelude as serenity;

/// Manage guild-specific settings.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("mod_role", "list", "color"),
    guild_only
)]
pub async fn settings(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Designate a role that can manage schedules.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn mod_role(
    ctx: Context<'_>,
    #[description = "Role to designate as scheduler moderator"] role: serenity::Role,
) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to manage settings.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?;
    ctx.data()
        .store
        .set_mod_role(guild_id.get(), role.id.get())
        .await?;

    ctx.say(format!("Moderator role set to @{}", role.name))
        .await?;
    Ok(())
}

/// Set the accent color for embeds in this server.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn color(
    ctx: Context<'_>,
    #[description = "Hex color code (e.g. 0xFF0000)"] color: String,
) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to manage settings.")
            .await?;
        return Ok(());
    }

    let color_val = if let Some(stripped) = color.strip_prefix("0x") {
        u32::from_str_radix(stripped, 16)
    } else if let Some(stripped) = color.strip_prefix('#') {
        u32::from_str_radix(stripped, 16)
    } else {
        u32::from_str_radix(&color, 16)
    };

    match color_val {
        Ok(c) => {
            let guild_id = ctx.guild_id().unwrap().get();
            ctx.data().store.set_accent_color(guild_id, c).await?;
            ctx.say(format!("Accent color updated to `0x{:06X}`.", c))
                .await?;
        }
        Err(_) => {
            ctx.say("Invalid color code. Use hex format like `0xABCDEF` or `#ABCDEF`.")
                .await?;
        }
    }
    Ok(())
}

/// Show current guild settings.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to view settings.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?;
    let settings = ctx.data().store.get_settings(guild_id.get()).await;

    let mod_role = match settings.mod_role_id {
        Some(id) => format!("<@&{}>", id),
        None => "Not set".to_string(),
    };

    let color = match settings.accent_color {
        Some(c) => format!("0x{:06X}", c),
        None => "Default (AniList Blue)".to_string(),
    };

    ctx.send(
        poise::CreateReply::default().embed(
            serenity::CreateEmbed::new()
                .title("Guild Settings")
                .field("Moderator Role", mod_role, true)
                .field("Accent Color", color, true),
        ),
    )
    .await?;

    Ok(())
}
