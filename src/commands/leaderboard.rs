use crate::models::bot_data::{Context, Error};
use crate::utils::embeds::leaderboard_embed;

/// Show the quiz leaderboard for this server.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?
        .get();
    let settings = ctx.data().store.get_settings(guild_id).await;

    ctx.send(poise::CreateReply::default().embed(leaderboard_embed(
        &settings.quiz_scores,
        settings.accent_color,
    )))
    .await?;
    Ok(())
}
