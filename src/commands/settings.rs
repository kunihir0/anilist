use poise::serenity_prelude as serenity;
use crate::models::bot_data::{Context, Error};

async fn check_admin(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Ok(false),
    };

    let member = match ctx.author_member().await {
        Some(m) => m,
        None => return Ok(false),
    };

    // Guild owners can always manage settings
    if let Some(owner_id) = ctx.cache().guild(guild_id).map(|g| g.owner_id) {
        if member.user.id == owner_id {
            return Ok(true);
        }
    }

    // Admins can always manage settings
    if member.permissions.map_or(false, |p| p.administrator()) {
        return Ok(true);
    }

    // Check for custom mod role
    if let Some(mod_role_id) = ctx.data().store.get_mod_role(guild_id.get()).await {
        if member.roles.contains(&serenity::RoleId::new(mod_role_id)) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Manage guild-specific settings.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("mod_role", "list"),
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
    if !check_admin(ctx).await? {
        ctx.say("You don't have permission to manage settings.").await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    ctx.data().store.set_mod_role(guild_id.get(), role.id.get()).await?;

    ctx.say(format!("Moderator role set to @{}", role.name)).await?;
    Ok(())
}

/// Show current guild settings.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    if !check_admin(ctx).await? {
        ctx.say("You don't have permission to view settings.").await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let settings = ctx.data().store.get_settings(guild_id.get()).await;

    let mod_role = match settings.mod_role_id {
        Some(id) => format!("<@&{}>", id),
        None => "Not set".to_string(),
    };

    ctx.send(poise::CreateReply::default().embed(
        serenity::CreateEmbed::new()
            .title("Guild Settings")
            .field("Moderator Role", mod_role, true)
    )).await?;

    Ok(())
}
