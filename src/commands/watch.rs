use crate::{
    api::anilist::{fetch_media_autocomplete, fetch_media_by_title},
    models::bot_data::{Context, Error},
    store::WatchParty,
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
    match fetch_media_autocomplete(&data.http_client, &data.rate_limiter, partial, "ANIME").await {
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

/// Watch party management.
#[poise::command(
    slash_command,
    prefix_command,
    subcommands("next", "vote"),
    guild_only,
    category = "Server"
)]
pub async fn watch(
    ctx: Context<'_>,
    #[description = "Media title to set"]
    #[autocomplete = "autocomplete_media"]
    title: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    match fetch_media_by_title(&data.http_client, &data.cache, &data.rate_limiter, &title).await {
        Ok(media) => {
            let party = WatchParty {
                media_id: media.id,
                title: media.title.preferred().to_string(),
                channel_id: ctx.channel_id().get(),
            };
            data.store
                .set_watch_party(
                    ctx.guild_id()
                        .ok_or("This command must be run in a server")?
                        .get(),
                    party,
                )
                .await?;
            ctx.say(format!(
                "Watch party series set to: **{}**",
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

/// Show information about the next episode of the current watch party series.
#[poise::command(slash_command, prefix_command)]
pub async fn next(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let settings = data
        .store
        .get_settings(
            ctx.guild_id()
                .ok_or("This command must be run in a server")?
                .get(),
        )
        .await;

    if let Some(party) = settings.watch_party {
        match fetch_media_by_title(
            &data.http_client,
            &data.cache,
            &data.rate_limiter,
            &party.title,
        )
        .await
        {
            Ok(media) => {
                if let Some(next) = media.next_airing_episode {
                    ctx.say(format!(
                        "Next episode of **{}**:\n**Episode {}** airs in **{}**.",
                        party.title,
                        next.episode,
                        next.countdown()
                    ))
                    .await?;
                } else {
                    ctx.say(format!(
                        "No upcoming airing episodes found for **{}**.",
                        party.title
                    ))
                    .await?;
                }
            }
            Err(e) => {
                ctx.say(format!("Error fetching media info: {}", e)).await?;
            }
        }
    } else {
        ctx.say("No watch party series is currently set. Use `/watch set <title>` first.")
            .await?;
    }
    Ok(())
}

/// Start a vote for the next watch party series.
#[poise::command(slash_command, prefix_command)]
pub async fn vote(
    ctx: Context<'_>,
    #[description = "Option 1"] opt1: String,
    #[description = "Option 2"] opt2: String,
    #[description = "Option 3"] opt3: Option<String>,
) -> Result<(), Error> {
    let mut options = vec![opt1, opt2];
    if let Some(o) = opt3 {
        options.push(o);
    }

    let description = options
        .iter()
        .enumerate()
        .map(|(i, o)| format!("{}. {}", i + 1, o))
        .collect::<Vec<_>>()
        .join("\n");

    let embed = serenity::CreateEmbed::new()
        .title("Watch Party Vote!")
        .description(format!(
            "React with the corresponding number to vote for the next series:\n\n{}",
            description
        ))
        .colour(serenity::Colour::new(0x3498db));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
