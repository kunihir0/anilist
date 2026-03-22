use crate::store::ServerListEntry;
use crate::{
    api::anilist::fetch_media_by_title,
    models::bot_data::{Context, Error},
    utils::embeds::server_list_embed,
};
use poise::serenity_prelude as serenity;

async fn autocomplete_media(
    ctx: Context<'_>,
    partial: &str,
) -> impl Iterator<Item = poise::serenity_prelude::AutocompleteChoice> {
    if partial.len() < 2 {
        return Vec::new().into_iter();
    }
    let data = ctx.data();
    match crate::api::anilist::fetch_media_autocomplete(
        &data.http_client,
        &data.rate_limiter,
        partial,
        "ANIME",
    )
    .await
    {
        Ok(items) => items
            .into_iter()
            .map(|m| {
                let label = match m.format.as_deref() {
                    Some(fmt) => format!("{} ({})", m.title.preferred(), fmt),
                    None => m.title.preferred().to_string(),
                };
                poise::serenity_prelude::AutocompleteChoice::new(
                    label,
                    m.title.preferred().to_string(),
                )
            })
            .collect::<Vec<_>>()
            .into_iter(),
        Err(_) => Vec::new().into_iter(),
    }
}

/// Manage the shared server anime list.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "list", "watched"),
    guild_only,
    category = "Server"
)]
pub async fn serverlist(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a title to the shared server list.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Title to add"]
    #[autocomplete = "autocomplete_media"]
    title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_media_by_title(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(media) => {
            let entry = ServerListEntry {
                id: rand::random::<u16>().to_string(),
                media_id: media.id,
                title: media.title.preferred().to_string(),
                added_by: ctx.author().id.get(),
                watched: false,
            };
            data.store
                .add_to_server_list(
                    ctx.guild_id()
                        .ok_or("This command must be run in a server")?
                        .get(),
                    entry,
                )
                .await?;
            ctx.say(format!(
                "Added **{}** to the server list.",
                media.title.preferred()
            ))
            .await?;
        }
        Err(e) => {
            ctx.say(format!("Could not find media: {}", e)).await?;
        }
    }
    Ok(())
}

/// List all titles in the shared server list.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?
        .get();
    let settings = ctx.data().store.get_settings(guild_id).await;

    ctx.send(poise::CreateReply::default().embed(server_list_embed(
        &settings.server_list,
        settings.accent_color,
    )))
    .await?;
    Ok(())
}

/// Mark a title as watched.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn watched(
    ctx: Context<'_>,
    #[description = "ID of the entry to mark watched"] id: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?
        .get();

    // Check for mod permissions
    let member = ctx.author_member().await.ok_or("Not in a guild")?;
    let mut is_mod = member.permissions.is_some_and(|p| p.administrator());

    if !is_mod
        && let Some(mod_role) = ctx.data().store.get_mod_role(guild_id).await
        && member.roles.contains(&serenity::RoleId::new(mod_role))
    {
        is_mod = true;
    }

    if !is_mod {
        ctx.say("You don't have permission to mark items as watched.")
            .await?;
        return Ok(());
    }

    if ctx.data().store.mark_watched(guild_id, &id).await? {
        ctx.say(format!("Marked entry `{}` as watched.", id))
            .await?;
    } else {
        ctx.say(format!("Entry `{}` not found.", id)).await?;
    }
    Ok(())
}
