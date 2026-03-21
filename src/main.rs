mod api;
mod commands;
mod models;
mod tasks;
mod utils;
mod store;

use models::bot_data::Data;
use api::cache::{Cache, RateLimiter};
use poise::serenity_prelude as serenity;
use reqwest::Client;
use tracing::{error, info};
use std::sync::Arc;
use store::Store;
use tokio_cron_scheduler::JobScheduler;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "anilist=info,poise=warn".into()),
        )
        .init();

    let _ = dotenvy::dotenv();

    let token = std::env::var("DISCORD_TOKEN")
        .expect("DISCORD_TOKEN is not set");

    let guild_id: Option<serenity::GuildId> = std::env::var("GUILD_ID")
        .ok()
        .and_then(|id| id.parse::<u64>().ok())
        .map(serenity::GuildId::new);

    let store = Arc::new(Store::new("store.json".into()).await.expect("Failed to initialize store"));
    let scheduler = JobScheduler::new().await.expect("Failed to create scheduler");
    let genres_cache = Arc::new(RwLock::new(Vec::new()));

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands::all(),
            on_error: |err| {
                Box::pin(async move {
                    match err {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            tracing::error!("Command '{}' failed: {error}", ctx.command().name);
                            let reply = poise::CreateReply::default()
                                .content(format!(
                                    "Something went wrong while running that command.\n```\n{error}\n```"
                                ))
                                .ephemeral(true);
                            let _ = ctx.send(reply).await;
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                tracing::error!("Error in builtins::on_error: {e}");
                            }
                        }
                    }
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            let genres_cache = genres_cache.clone();
            Box::pin(async move {
                match guild_id {
                    Some(gid) => {
                        poise::builtins::register_in_guild(
                            ctx, &framework.options().commands, gid,
                        ).await?;
                    }
                    None => {
                        poise::builtins::register_globally(
                            ctx, &framework.options().commands,
                        ).await?;
                    }
                }

                let cmd_names: Vec<String> = framework.options().commands.iter().map(|c| c.name.to_string()).collect();
                utils::startup::print_banner(&ready.user.name, ready.guilds.len(), &cmd_names);
                
                let client = Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .expect("Failed to build reqwest client");
                
                let cache = Cache::new(300);
                let rate_limiter = RateLimiter::new(90, 60);

                // Pre-fetch genres for autocomplete
                match api::anilist::fetch_genres(&client, &cache, &rate_limiter).await {
                    Ok(genres) => {
                        info!("Cached {} genres for autocomplete.", genres.len());
                        let mut g_lock = genres_cache.write().await;
                        *g_lock = genres;
                    }
                    Err(e) => {
                        error!("Failed to pre-fetch genres: {}", e);
                    }
                }

                let data = Data {
                    http_client: client,
                    cache,
                    rate_limiter,
                    store: store.clone(),
                    scheduler: scheduler.clone(),
                    genres: genres_cache,
                };

                tasks::presence::spawn(ctx.clone());
                tasks::scheduler::spawn_scheduler(ctx.clone(), Arc::new(data.clone())).await;

                Ok(data)
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create Serenity client");

    if let Err(e) = client.start().await {
        error!("Client encountered a fatal error: {e}");
    }
}
