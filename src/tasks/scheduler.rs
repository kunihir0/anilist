use crate::api::anilist::{
    fetch_airing, fetch_random, fetch_staff_birthdays, fetch_trending, fetch_upcoming,
};
use crate::models::bot_data::Data;
use crate::store::{ContentType, ScheduleEntry};
use crate::utils::embeds::{
    airing_page_embed, media_embed, media_list_embed, staff_birthday_embed, upcoming_page_embed,
};
use chrono::{Datelike, Utc};
use poise::serenity_prelude::{self as serenity, ChannelId};
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub async fn spawn_scheduler(ctx: serenity::Context, data: Arc<Data>) {
    let scheduler = data.scheduler.clone();

    // Initial registration of all active jobs from the store
    let schedules = data.store.get_all_schedules().await;
    for guild_schedules in schedules.values() {
        for entry in guild_schedules {
            if entry.active
                && let Err(e) = register_job(&scheduler, entry.clone(), &ctx, &data).await
            {
                tracing::error!("Failed to register job {}: {}", entry.id, e);
            }
        }
    }

    if let Err(e) = scheduler.start().await {
        tracing::error!("Failed to start scheduler: {}", e);
    }
}

pub async fn register_job(
    scheduler: &JobScheduler,
    entry: ScheduleEntry,
    ctx: &serenity::Context,
    data: &Arc<Data>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let http = ctx.http.clone();
    let data_clone = data.clone();
    let entry_clone = entry.clone();

    let job = Job::new_async(entry.cron_expression.as_str(), move |_uuid, _l| {
        let http = http.clone();
        let data = data_clone.clone();
        let entry = entry_clone.clone();
        Box::pin(async move {
            if let Err(e) = execute_job(http, data, entry).await {
                tracing::error!("Error executing scheduled job: {}", e);
            }
        })
    })?;

    scheduler.add(job).await?;
    Ok(())
}

async fn execute_job(
    http: Arc<serenity::Http>,
    data: Arc<Data>,
    entry: ScheduleEntry,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channel_id = ChannelId::new(entry.channel_id);
    let settings = data.store.get_settings(entry.guild_id).await;
    let accent_color = settings.accent_color;

    match entry.content_type {
        ContentType::DailyAnime => {
            let media = fetch_random(&data.http_client, &data.rate_limiter, "ANIME").await?;
            let embed = media_embed(&media, "Anime", None, accent_color);
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await?;
        }
        ContentType::DailyManga => {
            let media = fetch_random(&data.http_client, &data.rate_limiter, "MANGA").await?;
            let embed = media_embed(&media, "Manga", None, accent_color);
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await?;
        }
        ContentType::AiringUpdate => {
            let shows = fetch_airing(&data.http_client, &data.cache, &data.rate_limiter).await?;
            let embed = airing_page_embed(&shows, 1, 1, None, accent_color);
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await?;
        }
        ContentType::Trending => {
            let media =
                fetch_trending(&data.http_client, &data.cache, &data.rate_limiter, "ANIME").await?;
            let embed = media_list_embed(&media, "Trending Anime", None, accent_color);
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await?;
        }
        ContentType::NewSeason => {
            let month = Utc::now().month();
            let season = match month {
                3..=5 => "SPRING",
                6..=8 => "SUMMER",
                9..=11 => "FALL",
                _ => "WINTER",
            };
            let year = Utc::now().year();
            let shows = fetch_upcoming(
                &data.http_client,
                &data.cache,
                &data.rate_limiter,
                season,
                year,
            )
            .await?;
            let embed = upcoming_page_embed(&shows, season, year, 1, 1, None, accent_color);
            channel_id
                .send_message(&http, serenity::CreateMessage::new().embed(embed))
                .await?;
        }
        ContentType::StaffBirthday => {
            let staff =
                fetch_staff_birthdays(&data.http_client, &data.cache, &data.rate_limiter).await?;
            if !staff.is_empty() {
                let embed = staff_birthday_embed(&staff, accent_color);
                channel_id
                    .send_message(&http, serenity::CreateMessage::new().embed(embed))
                    .await?;
            }
        }
    }

    Ok(())
}
