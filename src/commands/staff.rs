use poise::CreateReply;
use crate::{
    api::anilist::{fetch_staff, fetch_staff_birthdays},
    models::bot_data::{Context, Error},
    utils::{
        embeds::{staff_embed, staff_birthday_embed},
        errors::reply_error,
    },
};

/// Search AniList for a staff member (VA, director, etc) by name.
#[poise::command(slash_command, prefix_command, subcommands("search", "today"))]
pub async fn staff(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Search AniList for a staff member by name.
#[poise::command(slash_command, prefix_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Staff name to search for"] name: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;

    match fetch_staff(&data.http_client, &data.cache, &data.rate_limiter, &name).await {
        Ok(staff) => {
            ctx.send(CreateReply::default().embed(staff_embed(&staff, prefs.title_language))).await?;
        }
        Err(e) => {
            tracing::warn!("Staff fetch failed for {name:?}: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}

/// See which staff members have birthdays today.
#[poise::command(slash_command, prefix_command)]
pub async fn today(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_staff_birthdays(&data.http_client, &data.cache, &data.rate_limiter).await {
        Ok(staff) => {
            ctx.send(CreateReply::default().embed(staff_birthday_embed(&staff))).await?;
        }
        Err(e) => {
            tracing::warn!("Staff birthdays fetch failed: {e}");
            reply_error(ctx, &e).await?;
        }
    }

    Ok(())
}
