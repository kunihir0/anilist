<div align="center">
  <img src="assets/logo.png" alt="Anilist Logo" width="150" />
  <h1>Anilist</h1>
  <p><strong>A Discord bot for querying AniList. Built with Rust, Poise, and Serenity.</strong></p>
</div>

## Commands

| Command | Description |
|---|---|
| `/anime <title>` | Search for an anime by title |
| `/manga <title>` | Search for a manga by title |
| `/profile <username>` | Look up an AniList user profile |

## Setup

**Prerequisites:** Rust toolchain, a Discord bot token, a bot application with `applications.commands` scope.

1. Clone the repo and copy the env file:
   ```
   cp .env.example .env
   ```

2. Fill in `.env`:
   ```
   DISCORD_TOKEN=your_token_here
   GUILD_ID=your_guild_id_here   # optional, remove for global registration
   ```

3. Run:
   ```
   cargo run
   ```

`GUILD_ID` registers commands instantly to one guild — useful for development. Without it, commands register globally and take up to one hour to propagate.

## Project Structure

```
src/
  main.rs              # Framework bootstrap
  commands/            # Slash command handlers
  api/                 # AniList GraphQL requests and query strings
  models/              # Serde structs and Poise type aliases
  tasks/               # Background tasks (rotating presence)
  utils/               # Embed builders and startup banner
```

## Dependencies

- [poise](https://github.com/serenity-rs/poise) — slash command framework
- [serenity](https://github.com/serenity-rs/serenity) — Discord API
- [reqwest](https://github.com/seanmonstar/reqwest) — HTTP client
- [tokio](https://tokio.rs) — async runtime
- [serde](https://serde.rs) — JSON deserialization