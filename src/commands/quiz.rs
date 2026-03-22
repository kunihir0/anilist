use crate::{
    api::anilist::fetch_random,
    models::bot_data::{Context, Error},
};
use futures::StreamExt;
use poise::serenity_prelude as serenity;

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum QuizDifficulty {
    #[name = "Easy"]
    Easy,
    #[name = "Medium"]
    Medium,
    #[name = "Hard"]
    Hard,
}

impl QuizDifficulty {
    pub fn get_rand_page(&self) -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let mut x: u64 = (dur.as_secs().wrapping_mul(1_000_000_007)) ^ (dur.subsec_nanos() as u64);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;

        let (min, max) = match self {
            Self::Easy => (1, 500u32),
            Self::Medium => (501, 2000),
            Self::Hard => (2001, 5000),
        };

        let range = max - min + 1;
        min + ((x % range as u64) as u32)
    }
}

/// Play a quick anime guessing quiz.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    user_cooldown = 10,
    category = "Server"
)]
pub async fn quiz(
    ctx: Context<'_>,
    #[description = "Difficulty level"] difficulty: Option<QuizDifficulty>,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a server")?
        .get();
    let settings = data.store.get_settings(guild_id).await;
    let accent_color = settings.accent_color;

    // Determine the page based on difficulty
    let page = difficulty.unwrap_or(QuizDifficulty::Medium).get_rand_page();

    // Fetch a random anime for the quiz
    let media = fetch_random(&data.http_client, &data.rate_limiter, "ANIME", Some(page)).await?;
    let target_title = media.title.preferred().to_string();
    let image_url = media
        .cover_image
        .as_ref()
        .and_then(|c| c.large.as_ref())
        .cloned()
        .unwrap_or_default();

    let embed = serenity::CreateEmbed::new()
        .title("Anime Quiz: Guess the Title!")
        .description(
            "You have 30 seconds to type the title in chat or click the button to reveal it.",
        )
        .image(image_url)
        .colour(accent_color.unwrap_or(0xFFA500));

    let ctx_id = ctx.id();
    let reveal_id = format!("{}reveal", ctx_id);
    let hint_id = format!("{}hint", ctx_id);

    let reply = poise::CreateReply::default().embed(embed).components(vec![
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&reveal_id)
                .label("Reveal Answer")
                .style(serenity::ButtonStyle::Danger),
            serenity::CreateButton::new(&hint_id)
                .label("Hint")
                .style(serenity::ButtonStyle::Primary),
        ]),
    ]);

    ctx.send(reply).await?;

    // Use cloned title for the filter closure
    let target_clone = target_title.to_lowercase();

    // We listen for messages in the same channel to see if anyone guesses correctly
    let mut message_collector = serenity::MessageCollector::new(ctx.serenity_context())
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(30))
        .stream();

    // Collector for the buttons
    let mut button_collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .filter(move |mci| mci.data.custom_id == reveal_id || mci.data.custom_id == hint_id)
        .timeout(std::time::Duration::from_secs(30))
        .stream();

    let mut participants = std::collections::HashSet::new();
    let start_time = std::time::Instant::now();

    loop {
        tokio::select! {
            Some(m) = message_collector.next() => {
                if m.author.bot { continue; }
                let user_id = m.author.id.get();
                participants.insert(user_id);

                let guess = m.content.to_lowercase();
                if guess.len() > 3 && (target_clone.contains(&guess) || guess.contains(&target_clone)) {
                    data.store.increment_quiz_score(guild_id, user_id, false).await?;

                    for p in participants {
                        if p != user_id {
                            let _ = data.store.increment_quiz_score(guild_id, p, true).await;
                        }
                    }

                    let win_embed = serenity::CreateEmbed::new()
                        .title("Correct!")
                        .description(format!("<@{}> guessed it correctly!\n\nThe answer was: **{}**\n[AniList Link]({})", user_id, target_title, media.site_url))
                        .colour(serenity::Colour::new(0x00FF00));

                    ctx.send(poise::CreateReply::default().embed(win_embed).components(vec![])).await?;
                    break;
                }
            }
            Some(mci) = button_collector.next() => {
                let custom_id = &mci.data.custom_id;

                if custom_id.ends_with("reveal") {
                    for p in participants {
                        let _ = data.store.increment_quiz_score(guild_id, p, true).await;
                    }

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
                    break;
                } else if custom_id.ends_with("hint") {
                    let elapsed = start_time.elapsed().as_secs();
                    let hint_text = if elapsed < 15 {
                        "Wait a bit longer for a hint (15s minimum for Genre, 20s for Year).".to_string()
                    } else if elapsed < 20 {
                        let genres = media.genres.join(", ");
                        format!("Hint (Genre): {}", if genres.is_empty() { "Unknown" } else { &genres })
                    } else {
                        let genres = media.genres.join(", ");
                        let year = media.season_year.map(|y| y.to_string()).unwrap_or_else(|| "Unknown".to_string());
                        format!("Hint (Genre & Year): {} | {}", if genres.is_empty() { "Unknown" } else { &genres }, year)
                    };

                    let _ = mci.create_response(
                        &ctx.serenity_context(),
                        serenity::CreateInteractionResponse::Message(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(hint_text)
                                .ephemeral(true)
                        )
                    ).await;
                }
            }
            else => {
                for p in participants {
                    let _ = data.store.increment_quiz_score(guild_id, p, true).await;
                }
                ctx.say(format!("Time's up! The answer was: **{}**", target_title)).await?;
                break;
            }
        }
    }

    Ok(())
}
