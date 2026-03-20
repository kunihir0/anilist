mod api;
mod commands;
mod models;
mod tasks;
mod utils;

use models::bot_data::Data;
use poise::serenity_prelude as serenity;
use reqwest::Client;
use tracing::error;

// ─── Entry Point ─────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Initialise the tracing subscriber so that poise::builtins::on_error can
    // log warnings and errors to stdout.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "anilist_bot=info,poise=warn".into()),
        )
        .init();

    // Load .env — silently ignore if there's no file (e.g. prod with real env vars)
    let _ = dotenvy::dotenv();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN is not set. Check your .env file.");

    // Optional: dev guild for instant slash command registration.
    let guild_id: Option<serenity::GuildId> = std::env::var("GUILD_ID")
        .ok()
        .and_then(|id| id.parse::<u64>().ok())
        .map(serenity::GuildId::new);

    // ── Framework ──────────────────────────────────────────────────────────
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),

            // ── Centralised error handler ───────────────────────────────────
            // poise::builtins::on_error logs the error via `tracing` and sends
            // a friendly Discord message to the user when applicable.
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            tracing::error!("Command '{}' failed: {error}", ctx.command().name);
                            let reply = poise::CreateReply::default()
                                .content(format!(
                                    "⚠️ Something went wrong while running that command.\n```\n{error}\n```"
                                ))
                                .ephemeral(true);
                            let _ = ctx.send(reply).await;
                        }
                        other => {
                            // Delegate everything else (setup, event, cooldown, permissions…)
                            // to the built-in Poise handler which logs via tracing.
                            if let Err(e) = poise::builtins::on_error(other).await {
                                tracing::error!("Error in builtins::on_error: {e}");
                            }
                        }
                    }
                })
            },

            ..Default::default()
        })
        // ── Setup callback — called once when the bot connects ──────────────
        .setup(move |ctx, ready, framework| {
            Box::pin(async move {
                // Register slash commands either to a single dev guild
                // (instant) or globally (up to 1 hour propagation).
                match guild_id {
                    Some(gid) => {
                        poise::builtins::register_in_guild(
                            ctx,
                            &framework.options().commands,
                            gid,
                        )
                        .await?;
                    }
                    None => {
                        poise::builtins::register_globally(
                            ctx,
                            &framework.options().commands,
                        )
                        .await?;
                    }
                }

                // ── Startup banner (colored console output) ─────────────────
                utils::startup::print_banner(
                    &ready.user.name,
                    ready.guilds.len(),
                );

                // ── Rotating presence ────────────────────────────────────────
                // Spawn a background tokio task that cycles through 10 statuses,
                // updating every 30 seconds.  Cloning ctx is cheap — it's an
                // Arc wrapper around the underlying shard manager.
                tasks::presence::spawn(ctx.clone());

                // Build the shared Data struct with a single long-lived HTTP client.
                Ok(Data {
                    http_client: Client::builder()
                        .timeout(std::time::Duration::from_secs(10))
                        .build()
                        .expect("Failed to build reqwest client"),
                })
            })
        })
        .build();

    // ── Serenity client ────────────────────────────────────────────────────
    // We only need non-privileged intents because this bot never reads message
    // content — it responds only to slash commands and prefix commands triggered
    // by the bot itself.
    let intents = serenity::GatewayIntents::non_privileged();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create Serenity client");

    if let Err(e) = client.start().await {
        error!("Client encountered a fatal error: {e}");
    }
}