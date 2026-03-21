use std::time::Duration;

use futures::StreamExt;
use poise::CreateReply;
use poise::serenity_prelude::{
    self as serenity, ButtonStyle, CreateActionRow, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage,
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
