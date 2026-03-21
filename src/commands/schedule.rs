use crate::models::bot_data::{Context, Error};
use crate::store::{ContentType, ScheduleEntry};
use crate::tasks::scheduler::register_job;
use crate::utils::permissions::check_admin_or_mod;
use poise::serenity_prelude as serenity;
use std::sync::Arc;

#[derive(Debug, poise::ChoiceParameter, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleContentType {
    #[name = "daily-anime"]
    DailyAnime,
    #[name = "daily-manga"]
    DailyManga,
    #[name = "airing-update"]
    AiringUpdate,
    #[name = "trending"]
    Trending,
    #[name = "new-season"]
    NewSeason,
    #[name = "staff-birthday"]
    StaffBirthday,
}

impl From<ScheduleContentType> for ContentType {
    fn from(s: ScheduleContentType) -> Self {
        match s {
            ScheduleContentType::DailyAnime => ContentType::DailyAnime,
            ScheduleContentType::DailyManga => ContentType::DailyManga,
            ScheduleContentType::AiringUpdate => ContentType::AiringUpdate,
            ScheduleContentType::Trending => ContentType::Trending,
            ScheduleContentType::NewSeason => ContentType::NewSeason,
            ScheduleContentType::StaffBirthday => ContentType::StaffBirthday,
        }
    }
}

/// Manage automated AniList schedules for this guild.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "remove", "list", "pause"),
    guild_only
)]
pub async fn schedule(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a new automated schedule entry.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Channel to post in"] channel: serenity::Channel,
    #[description = "Content to post"] content: ScheduleContentType,
    #[description = "Cron expression (e.g. '0 9 * * *' for daily at 9am)"] cron: String,
    #[description = "Timezone (e.g. 'UTC')"] timezone: Option<String>,
) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to manage schedules.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?;
    let id = rand::random::<u32>().to_string(); // Simple ID generation

    let entry = ScheduleEntry {
        id: id.clone(),
        guild_id: guild_id.get(),
        channel_id: channel.id().get(),
        content_type: content.into(),
        cron_expression: cron,
        timezone: timezone.unwrap_or_else(|| "UTC".to_string()),
        active: true,
    };

    // Try to register with the live scheduler first to validate the cron string
    let data = ctx.data();
    let data_arc = Arc::new(data.clone());
    if let Err(e) = register_job(
        &data.scheduler,
        entry.clone(),
        ctx.serenity_context(),
        &data_arc,
    )
    .await
    {
        ctx.say(format!(
            "Failed to add schedule: Invalid cron expression? ({})",
            e
        ))
        .await?;
        return Ok(());
    }

    data.store.add_schedule(entry).await?;
    ctx.say(format!(
        "Added schedule with ID `{}` for channel <#{}>.",
        id,
        channel.id()
    ))
    .await?;
    Ok(())
}

/// Remove a schedule entry by its ID.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "ID of the schedule to remove"] id: String,
) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to manage schedules.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?;
    if ctx
        .data()
        .store
        .remove_schedule(guild_id.get(), &id)
        .await?
    {
        ctx.say(format!("Removed schedule `{}`.", id)).await?;
    } else {
        ctx.say(format!("Schedule `{}` not found in this guild.", id))
            .await?;
    }
    Ok(())
}

/// List all active schedules for this guild.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to view schedules.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?;
    let schedules = ctx.data().store.list_schedules(guild_id.get()).await;

    if schedules.is_empty() {
        ctx.say("No schedules configured for this guild.").await?;
        return Ok(());
    }

    let mut embed = serenity::CreateEmbed::new().title("Guild Schedules");
    for s in schedules {
        let status = if s.active {
            "✅ Active"
        } else {
            "⏸️ Paused"
        };
        let value = format!(
            "Channel: <#{}>\nContent: `{}`\nCron: `{}`\nTimezone: `{}`\nStatus: {}",
            s.channel_id, s.content_type, s.cron_expression, s.timezone, status
        );
        embed = embed.field(format!("ID: {}", s.id), value, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Temporarily pause or resume a schedule without deleting it.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn pause(
    ctx: Context<'_>,
    #[description = "ID of the schedule to toggle"] id: String,
) -> Result<(), Error> {
    if !check_admin_or_mod(ctx).await? {
        ctx.say("You don't have permission to manage schedules.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?;
    match ctx
        .data()
        .store
        .toggle_schedule(guild_id.get(), &id)
        .await?
    {
        Some(true) => {
            ctx.say(format!("Schedule `{}` is now **active**.", id))
                .await?;
        }
        Some(false) => {
            ctx.say(format!("Schedule `{}` is now **paused**.", id))
                .await?;
        }
        None => {
            ctx.say(format!("Schedule `{}` not found.", id)).await?;
        }
    }
    Ok(())
}
