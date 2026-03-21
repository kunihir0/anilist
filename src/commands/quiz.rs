use poise::serenity_prelude as serenity;
use crate::{
    api::anilist::fetch_random,
    models::bot_data::{Context, Error},
};
use futures::StreamExt;

/// Play a quick anime guessing quiz.
#[poise::command(slash_command, prefix_command, guild_only)]
pub async fn quiz(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let guild_id = ctx.guild_id().ok_or("This command must be run in a server")?.get();
    let settings = data.store.get_settings(guild_id).await;
    let accent_color = settings.accent_color;

    // Fetch a random anime for the quiz
    let media = fetch_random(&data.http_client, &data.rate_limiter, "ANIME").await?;
    let target_title = media.title.preferred().to_string();
    let image_url = media.cover_image.as_ref().and_then(|c| c.large.as_ref()).cloned().unwrap_or_default();

    let embed = serenity::CreateEmbed::new()
        .title("Anime Quiz: Guess the Title!")
        .description("You have 30 seconds to type the title in chat or click the button to reveal it.")
        .image(image_url)
        .colour(accent_color.unwrap_or(0xFFA500));

    let ctx_id = ctx.id();
    let button_id = format!("{}reveal", ctx_id);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&button_id).label("Reveal Answer").style(serenity::ButtonStyle::Primary)
        ])]);

    ctx.send(reply).await?;

    // Use cloned title for the filter closure
    let target_clone = target_title.to_lowercase();

    // We also listen for messages in the same channel to see if anyone guesses correctly
    let mut message_collector = serenity::MessageCollector::new(ctx.serenity_context())
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(30))
        .filter(move |m| {
            let guess = m.content.to_lowercase();
            guess.len() > 3 && (target_clone.contains(&guess) || guess.contains(&target_clone))
        })
        .stream();

    // Collector for the button
    let mut button_collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .filter(move |mci| mci.data.custom_id == button_id)
        .timeout(std::time::Duration::from_secs(30))
        .stream();

    tokio::select! {
        Some(m) = message_collector.next() => {
            let user_id = m.author.id.get();
            data.store.increment_quiz_score(guild_id, user_id).await?;

            let win_embed = serenity::CreateEmbed::new()
                .title("Correct!")
                .description(format!("<@{}> guessed it correctly!\n\nThe answer was: **{}**\n[AniList Link]({})", user_id, target_title, media.site_url))
                .colour(serenity::Colour::new(0x00FF00));

            ctx.send(poise::CreateReply::default().embed(win_embed).components(vec![])).await?;
        }
        Some(mci) = button_collector.next() => {
            let reveal_embed = serenity::CreateEmbed::new()
                .title("Quiz Answer")
                .description(format!("The answer was: **{}**\n[AniList Link]({})", target_title, media.site_url))
                .colour(serenity::Colour::new(0x00FF00));

            mci.create_response(
                &ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(reveal_embed)
                        .components(vec![])
                )
            ).await?;
        }
        else => {
            ctx.say(format!("Time's up! The answer was: **{}**", target_title)).await?;
        }
    }

    Ok(())
}
