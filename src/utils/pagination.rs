use std::time::Duration;

use futures::StreamExt;
use poise::CreateReply;
use poise::serenity_prelude::{
    self as serenity, ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage,
};
use serenity::collector::ComponentInteractionCollector;

use crate::models::bot_data::{Context, Error};

// ─── Button builders ──────────────────────────────────────────────────────────

fn nav_row(index: usize, total: usize) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("prev")
            .label("◀  Prev")
            .style(ButtonStyle::Secondary)
            .disabled(index == 0),
        CreateButton::new("page_count")
            .label(format!("{} / {}", index + 1, total))
            .style(ButtonStyle::Secondary)
            .disabled(true),
        CreateButton::new("next")
            .label("Next  ▶")
            .style(ButtonStyle::Primary)
            .disabled(index + 1 >= total),
    ])
}

fn disabled_row(index: usize, total: usize) -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("prev")
            .label("◀  Prev")
            .style(ButtonStyle::Secondary)
            .disabled(true),
        CreateButton::new("page_count")
            .label(format!("{} / {}", index + 1, total))
            .style(ButtonStyle::Secondary)
            .disabled(true),
        CreateButton::new("next")
            .label("Next  ▶")
            .style(ButtonStyle::Primary)
            .disabled(true),
    ])
}

fn media_interactive_row() -> CreateActionRow {
    CreateActionRow::Buttons(vec![
        CreateButton::new("media_characters")
            .label("👥 Characters")
            .style(ButtonStyle::Secondary),
        CreateButton::new("media_relations")
            .label("🔗 Relations")
            .style(ButtonStyle::Secondary),
        CreateButton::new("media_recommendations")
            .label("⭐ Recommendations")
            .style(ButtonStyle::Secondary),
    ])
}

// ─── Public paginator ─────────────────────────────────────────────────────────

/// Send a paginated response.
///
/// - If `pages` is empty, does nothing.
/// - If `pages` has one item, sends it without any buttons.
/// - Otherwise sends the first embed with Prev / counter / Next buttons and
///   collects interactions for 120 seconds. On timeout the buttons are greyed
///   out. Only the original command author can navigate.
pub async fn paginate(ctx: Context<'_>, pages: Vec<serenity::CreateEmbed>) -> Result<(), Error> {
    if pages.is_empty() {
        return Ok(());
    }

    if pages.len() == 1 {
        ctx.send(CreateReply::default().embed(pages.into_iter().next().unwrap()))
            .await?;
        return Ok(());
    }

    let total = pages.len();
    let mut index: usize = 0;
    let author_id = ctx.author().id;

    let reply = ctx
        .send(
            CreateReply::default()
                .embed(pages[index].clone())
                .components(vec![nav_row(index, total)]),
        )
        .await?;

    let message = reply.into_message().await?;

    let mut stream = ComponentInteractionCollector::new(ctx.serenity_context())
        .message_id(message.id)
        .author_id(author_id)
        .timeout(Duration::from_secs(120))
        .stream();

    while let Some(interaction) = stream.next().await {
        match interaction.data.custom_id.as_str() {
            "prev" => {
                index = index.saturating_sub(1);
            }
            "next" => {
                if index + 1 < total {
                    index += 1;
                }
            }
            _ => continue,
        }

        interaction
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(pages[index].clone())
                        .components(vec![nav_row(index, total)]),
                ),
            )
            .await?;
    }

    // Timeout — disable buttons so the embed doesn't look broken.
    message
        .clone()
        .edit(
            ctx.serenity_context(),
            serenity::EditMessage::new().components(vec![disabled_row(index, total)]),
        )
        .await?;

    Ok(())
}

/// Send a paginated response for media, including interactive buttons for characters/relations/recommendations.
pub async fn paginate_media(
    ctx: Context<'_>,
    pages: Vec<(serenity::CreateEmbed, crate::models::responses::Media)>,
) -> Result<(), Error> {
    if pages.is_empty() {
        return Ok(());
    }

    if pages.len() == 1 {
        let (embed, _) = &pages[0];
        ctx.send(
            CreateReply::default()
                .embed(embed.clone())
                .components(vec![media_interactive_row()]),
        )
        .await?;
        // We still need to collect interactions for the single page
    }

    let total = pages.len();
    let mut index: usize = 0;
    let author_id = ctx.author().id;

    let reply = if total > 1 {
        ctx.send(
            CreateReply::default()
                .embed(pages[index].0.clone())
                .components(vec![nav_row(index, total), media_interactive_row()]),
        )
        .await?
    } else {
        // We already sent above, so we get the message. Wait, let's just unify.
        ctx.send(
            CreateReply::default()
                .embed(pages[index].0.clone())
                .components(vec![media_interactive_row()]),
        )
        .await?
    };

    let message = reply.into_message().await?;

    let mut stream = ComponentInteractionCollector::new(ctx.serenity_context())
        .message_id(message.id)
        .author_id(author_id)
        .timeout(Duration::from_secs(120))
        .stream();

    while let Some(interaction) = stream.next().await {
        match interaction.data.custom_id.as_str() {
            "prev" => {
                index = index.saturating_sub(1);
            }
            "next" => {
                if index + 1 < total {
                    index += 1;
                }
            }
            "media_characters" => {
                interaction.defer_ephemeral(ctx.serenity_context()).await?;
                let media_id = pages[index].1.id;
                let data = ctx.data();
                match crate::api::anilist::fetch_media_characters_by_id(
                    &data.http_client,
                    &data.cache,
                    &data.rate_limiter,
                    media_id,
                )
                .await
                {
                    Ok(media) => {
                        let mut desc = String::new();
                        if let Some(chars) = media.characters {
                            for edge in chars.edges.iter().take(15) {
                                let role = edge.role.as_deref().unwrap_or("Unknown");
                                desc.push_str(&format!(
                                    "**{}** ({role})\n",
                                    edge.node.name.preferred()
                                ));
                            }
                            if desc.is_empty() {
                                desc = "No characters found.".to_string();
                            }
                        } else {
                            desc = "No characters found.".to_string();
                        }
                        let e = serenity::CreateEmbed::default()
                            .title("Characters")
                            .description(desc)
                            .color(0x3498db);
                        interaction
                            .create_followup(
                                ctx.serenity_context(),
                                CreateInteractionResponseFollowup::new()
                                    .embed(e)
                                    .ephemeral(true),
                            )
                            .await?;
                    }
                    Err(_) => {
                        interaction
                            .create_followup(
                                ctx.serenity_context(),
                                CreateInteractionResponseFollowup::new()
                                    .content("Failed to fetch characters.")
                                    .ephemeral(true),
                            )
                            .await?;
                    }
                }
                continue;
            }
            "media_relations" => {
                let media = &pages[index].1;
                let data = ctx.data();
                let prefs = data.store.get_user_prefs(ctx.author().id.get()).await;
                let guild_id = ctx.guild_id().map(|id| id.get());
                let accent_color = if let Some(gid) = guild_id {
                    data.store.get_settings(gid).await.accent_color
                } else {
                    None
                };
                let e = crate::utils::embeds::relations_embed(
                    media,
                    prefs.title_language,
                    accent_color,
                );
                interaction
                    .create_response(
                        ctx.serenity_context(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .embed(e)
                                .ephemeral(true),
                        ),
                    )
                    .await?;
                continue;
            }
            "media_recommendations" => {
                interaction.defer_ephemeral(ctx.serenity_context()).await?;
                let media_id = pages[index].1.id;
                let data = ctx.data();
                match crate::api::anilist::fetch_media_recommendations_by_id(
                    &data.http_client,
                    &data.cache,
                    &data.rate_limiter,
                    media_id,
                )
                .await
                {
                    Ok(media) => {
                        let mut desc = String::new();
                        if let Some(recs) = media.recommendations {
                            for node in recs.nodes.iter().take(5) {
                                if let Some(rec) = &node.media_recommendation {
                                    desc.push_str(&format!(
                                        "• [{}]({})\n",
                                        rec.title.preferred(),
                                        rec.site_url
                                    ));
                                }
                            }
                            if desc.is_empty() {
                                desc = "No recommendations found.".to_string();
                            }
                        } else {
                            desc = "No recommendations found.".to_string();
                        }
                        let e = serenity::CreateEmbed::default()
                            .title("Recommendations")
                            .description(desc)
                            .color(0xe67e22);
                        interaction
                            .create_followup(
                                ctx.serenity_context(),
                                CreateInteractionResponseFollowup::new()
                                    .embed(e)
                                    .ephemeral(true),
                            )
                            .await?;
                    }
                    Err(_) => {
                        interaction
                            .create_followup(
                                ctx.serenity_context(),
                                CreateInteractionResponseFollowup::new()
                                    .content("Failed to fetch recommendations.")
                                    .ephemeral(true),
                            )
                            .await?;
                    }
                }
                continue;
            }
            _ => continue,
        }

        let mut components = vec![];
        if total > 1 {
            components.push(nav_row(index, total));
        }
        components.push(media_interactive_row());

        interaction
            .create_response(
                ctx.serenity_context(),
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new()
                        .embed(pages[index].0.clone())
                        .components(components),
                ),
            )
            .await?;
    }

    let mut disable_components = vec![];
    if total > 1 {
        disable_components.push(disabled_row(index, total));
    }
    // Also disable media interactive buttons if we want, or just remove them
    disable_components.push(CreateActionRow::Buttons(vec![
        CreateButton::new("media_characters")
            .label("👥 Characters")
            .style(ButtonStyle::Secondary)
            .disabled(true),
        CreateButton::new("media_relations")
            .label("🔗 Relations")
            .style(ButtonStyle::Secondary)
            .disabled(true),
        CreateButton::new("media_recommendations")
            .label("⭐ Recommendations")
            .style(ButtonStyle::Secondary)
            .disabled(true),
    ]));

    message
        .clone()
        .edit(
            ctx.serenity_context(),
            serenity::EditMessage::new().components(disable_components),
        )
        .await?;

    Ok(())
}
