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

// ─── Title matching helpers ──────────────────────────────────────────────────

/// Normalize a string for fuzzy comparison: lowercase, strip punctuation, collapse whitespace.
fn normalize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == ' ' {
                c
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Check if guess matches any of the known titles for a media.
fn is_correct_guess(
    guess: &str,
    romaji: Option<&str>,
    english: Option<&str>,
    native: Option<&str>,
) -> bool {
    let norm_guess = normalize(guess);
    if norm_guess.len() < 3 {
        return false;
    }

    // Collect all valid title variants
    let titles: Vec<String> = [romaji, english, native]
        .iter()
        .filter_map(|t| t.map(normalize))
        .filter(|t| !t.is_empty() && t != "unknown title")
        .collect();

    for title in &titles {
        // Exact match after normalization
        if norm_guess == *title {
            return true;
        }
        // Guess is a substantial substring of the title (≥60% of the title length)
        if title.contains(&norm_guess) && norm_guess.len() * 100 / title.len().max(1) >= 60 {
            return true;
        }
        // Title is contained within the guess (user typed extra words around it)
        if norm_guess.contains(title.as_str()) {
            return true;
        }
        // Word-level similarity: check if most words of the title appear in the guess
        let title_words: Vec<&str> = title.split_whitespace().collect();
        let guess_words: Vec<&str> = norm_guess.split_whitespace().collect();
        if title_words.len() >= 2 {
            let matched = title_words
                .iter()
                .filter(|tw| guess_words.iter().any(|gw| *gw == **tw))
                .count();
            // If ≥75% of the title words are in the guess, accept it
            if matched * 100 / title_words.len() >= 75 {
                return true;
            }
        }
    }

    false
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
    let diff = difficulty.unwrap_or(QuizDifficulty::Medium);
    let page = diff.get_rand_page();

    // Fetch a random anime for the quiz
    let media = fetch_random(&data.http_client, &data.rate_limiter, "ANIME", Some(page)).await?;
    let target_title = media.title.preferred().to_string();
    let image_url = media
        .cover_image
        .as_ref()
        .and_then(|c| c.large.as_ref())
        .cloned()
        .unwrap_or_default();

    // Clone title variants for matching
    let romaji = media.title.romaji.clone();
    let english = media.title.english.clone();
    let native = media.title.native.clone();

    let diff_label = match diff {
        QuizDifficulty::Easy => "Easy 🟢",
        QuizDifficulty::Medium => "Medium 🟡",
        QuizDifficulty::Hard => "Hard 🔴",
    };

    let embed = serenity::CreateEmbed::new()
        .title(format!("Anime Quiz — {}", diff_label))
        .description(
            "Guess the anime title! Type your answer in chat.\n\
             You have **30 seconds**. Use the buttons below for hints or to give up.",
        )
        .image(image_url)
        .colour(accent_color.unwrap_or(0xFFA500))
        .footer(serenity::CreateEmbedFooter::new(
            "Tip: English or romaji titles both work!",
        ));

    let ctx_id = ctx.id();
    let reveal_id = format!("{}reveal", ctx_id);
    let hint_id = format!("{}hint", ctx_id);

    let reply = poise::CreateReply::default().embed(embed).components(vec![
        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&reveal_id)
                .label("Give Up")
                .style(serenity::ButtonStyle::Danger),
            serenity::CreateButton::new(&hint_id)
                .label("Hint")
                .style(serenity::ButtonStyle::Secondary),
        ]),
    ]);

    ctx.send(reply).await?;

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
    let mut hint_stage = 0u8; // 0 = no hint shown, 1 = genre, 2 = genre + year

    loop {
        tokio::select! {
            Some(m) = message_collector.next() => {
                if m.author.bot { continue; }
                let user_id = m.author.id.get();
                participants.insert(user_id);

                if is_correct_guess(
                    &m.content,
                    romaji.as_deref(),
                    english.as_deref(),
                    native.as_deref(),
                ) {
                    data.store.increment_quiz_score(guild_id, user_id, false).await?;

                    // Break streaks for everyone else who participated
                    for &p in &participants {
                        if p != user_id {
                            let _ = data.store.increment_quiz_score(guild_id, p, true).await;
                        }
                    }

                    let elapsed = start_time.elapsed().as_secs();
                    let speed_bonus = if elapsed <= 5 { " ⚡ Lightning fast!" } else if elapsed <= 15 { " 🔥 Quick!" } else { "" };

                    let win_embed = serenity::CreateEmbed::new()
                        .title("✅ Correct!")
                        .description(format!(
                            "<@{}> guessed it in **{}s**!{}\n\nThe answer was: **{}**\n[View on AniList]({})",
                            user_id, elapsed, speed_bonus, target_title, media.site_url
                        ))
                        .colour(serenity::Colour::new(0x2ECC71))
                        .thumbnail(
                            media.cover_image.as_ref()
                                .and_then(|c| c.large.as_ref())
                                .cloned()
                                .unwrap_or_default()
                        );

                    ctx.send(poise::CreateReply::default().embed(win_embed).components(vec![])).await?;
                    break;
                }
            }
            Some(mci) = button_collector.next() => {
                let custom_id = &mci.data.custom_id;

                if custom_id.ends_with("reveal") {
                    // Break streaks for all participants
                    for &p in &participants {
                        let _ = data.store.increment_quiz_score(guild_id, p, true).await;
                    }

                    let reveal_embed = serenity::CreateEmbed::new()
                        .title("💡 Quiz Answer")
                        .description(format!(
                            "Nobody got it! The answer was: **{}**\n[View on AniList]({})",
                            target_title, media.site_url
                        ))
                        .colour(serenity::Colour::new(0xE74C3C));

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
                    let hint_text = if elapsed < 10 && hint_stage == 0 {
                        "⏳ Wait at least 10 seconds before getting a hint!".to_string()
                    } else if hint_stage == 0 {
                        hint_stage = 1;
                        let genres = media.genres.join(", ");
                        format!(
                            "🔍 **Genre:** {}",
                            if genres.is_empty() { "Unknown".to_string() } else { genres }
                        )
                    } else if hint_stage == 1 {
                        hint_stage = 2;
                        let year = media.season_year
                            .map(|y| y.to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        let eps = media.episodes
                            .map(|e| format!("{} episodes", e))
                            .unwrap_or_else(|| "Unknown length".to_string());
                        format!("🔍 **Year:** {} | **Length:** {}", year, eps)
                    } else {
                        // Third hint: first letter
                        let first_char = target_title.chars().next().unwrap_or('?');
                        format!("🔍 **Starts with:** {}", first_char)
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
                // Time's up
                for &p in &participants {
                    let _ = data.store.increment_quiz_score(guild_id, p, true).await;
                }

                let timeout_embed = serenity::CreateEmbed::new()
                    .title("⏰ Time's Up!")
                    .description(format!(
                        "The answer was: **{}**\n[View on AniList]({})",
                        target_title, media.site_url
                    ))
                    .colour(serenity::Colour::new(0xE74C3C));

                ctx.send(poise::CreateReply::default().embed(timeout_embed).components(vec![])).await?;
                break;
            }
        }
    }

    Ok(())
}
