use poise::serenity_prelude as serenity;
use crate::{
    api::anilist::fetch_random,
    models::bot_data::{Context, Error},
};
use futures::StreamExt;

/// Play a quick anime guessing quiz.
#[poise::command(slash_command, prefix_command)]
pub async fn quiz(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();

    // Fetch a random anime for the quiz
    let media = fetch_random(&data.http_client, &data.rate_limiter, "ANIME").await?;
    let title = media.title.preferred();
    let image_url = media.cover_image.as_ref().and_then(|c| c.large.as_ref()).cloned().unwrap_or_default();

    let embed = serenity::CreateEmbed::new()
        .title("Anime Quiz: Guess the Title!")
        .description("You have 30 seconds to type the title in chat or click the button to reveal it.")
        .image(image_url)
        .colour(serenity::Colour::new(0xFFA500));

    let ctx_id = ctx.id();
    let button_id = format!("{}reveal", ctx_id);

    let reply = poise::CreateReply::default()
        .embed(embed)
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&button_id).label("Reveal Answer").style(serenity::ButtonStyle::Primary)
        ])]);

    ctx.send(reply).await?;

    // Collector for the button
    let mut collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .filter(move |mci| mci.data.custom_id == button_id)
        .timeout(std::time::Duration::from_secs(30))
        .build();

    if let Some(mci) = collector.next().await {
        let reveal_embed = serenity::CreateEmbed::new()
            .title("Quiz Answer")
            .description(format!("The answer was: **{}**\n[AniList Link]({})", title, media.site_url))
            .colour(serenity::Colour::new(0x00FF00));

        mci.create_response(
            &ctx.serenity_context(),
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(reveal_embed)
                    .components(vec![]) // Remove button
            )
        ).await?;
    } else {
        // Timeout
        ctx.say(format!("Time's up! The answer was: **{}**", title)).await?;
    }

    Ok(())
}
