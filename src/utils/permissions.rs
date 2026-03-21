use crate::models::bot_data::{Context, Error};
use poise::serenity_prelude as serenity;

/// Checks if the current user has permission to manage guild settings or schedules.
/// This is true if the user is the guild owner, has the Administrator permission,
/// or possesses the designated moderator role.
pub async fn check_admin_or_mod(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => return Ok(false),
    };

    let member = match ctx.author_member().await {
        Some(m) => m,
        None => return Ok(false),
    };

    // Guild owners can always manage settings
    if let Some(owner_id) = ctx.cache().guild(guild_id).map(|g| g.owner_id)
        && member.user.id == owner_id
    {
        return Ok(true);
    }

    // Admins can always manage settings
    if member.permissions.is_some_and(|p| p.administrator()) {
        return Ok(true);
    }

    // Check for custom mod role
    if let Some(mod_role_id) = ctx.data().store.get_mod_role(guild_id.get()).await
        && member.roles.contains(&serenity::RoleId::new(mod_role_id))
    {
        return Ok(true);
    }

    Ok(false)
}
