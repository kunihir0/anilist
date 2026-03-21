<div align="center">
  <img src="assets/logo.png" alt="Anilist Logo" width="150" />
  <h1>Anilist</h1>
  <p><strong>A Discord bot for querying AniList. Built with Rust, Poise, and Serenity.</strong></p>
  <a href="https://discord.com/oauth2/authorize?client_id=1484531884628246569&scope=bot+applications.commands&permissions=2147483648">
    <img src="https://img.shields.io/badge/Add%20to%20Discord-5865F2?style=for-the-badge&logo=discord&logoColor=white" alt="Add to Discord" />
  </a>
</div>

&nbsp;

[![CI](https://github.com/kunihir0/anilist/actions/workflows/ci.yml/badge.svg)](https://github.com/kunihir0/anilist/actions/workflows/ci.yml)
[![Docker](https://github.com/kunihir0/anilist/actions/workflows/docker.yml/badge.svg)](https://github.com/kunihir0/anilist/actions/workflows/docker.yml)

## Commands

| Command | Description |
|---|---|
| `/anime <title>` | Search for an anime by title — paginated, up to 5 results |
| `/manga <title>` | Search for a manga by title — paginated, up to 5 results |
| `/character <name>` | Look up a character and their media appearances |
| `/staff <name>` | Look up a voice actor, director, or composer |
| `/studio <name>` | Look up a studio and their notable works |
| `/profile <username>` | Look up an AniList user profile and stats |
| `/compare <user1> <user2>` | Compare two AniList profiles side by side |
| `/favourites <username>` | Show a user's public AniList favourites |
| `/watchlist <username>` | Browse a user's anime or manga list by status |
| `/recommendations <title>` | Get community recommendations for a title |
| `/relations <title>` | Show sequels, prequels, and related entries |
| `/trending` | Top trending anime or manga right now |
| `/upcoming` | Seasonal anime chart — defaults to current season |
| `/airing` | Currently airing anime with episode countdowns |
| `/random` | Get a random well-rated anime or manga |
| `/genre <genre>` | Browse top titles by genre (with autocomplete) |
| `/filter` | Advanced search by format, status, year, country, and more |
| `/quiz` | Play a quick anime guessing quiz |
| `/watch <set|next|vote>` | Manage a channel's watch party series |
| `/serverlist <add|list|watched>` | Manage a shared server anime list |
| `/schedule <add|list|pause|remove>` | Set up automated daily/weekly AniList posts |
| `/settings <mod_role|color|list>` | Configure guild settings and embed themes |
| `/leaderboard` | View the server's `/quiz` leaderboard |
| `/recommend compare <u1> <u2>` | Find shows one user has seen that the other hasn't |
| `/prefs title_language` | Set your preferred title language (Romaji/English/Native) |
| `/ping` | Check bot latency and API response time |
| `/help` | List all available commands |

## Setup

**Prerequisites:** A Discord bot token and a bot application with `applications.commands` scope.

### Local

Requires the Rust toolchain.

1. Clone the repo and copy the env file:
   ```
   cp .env.example .env
   ```

2. Fill in `.env`:
   ```
   DISCORD_TOKEN=your_token_here
   GUILD_ID=your_guild_id_here   # optional, remove for global registration
   DATABASE_URL=sqlite:anilist.db?mode=rwc # optional, defaults to this
   ```

3. Run:
   ```
   cargo run
   ```

### Docker

**From GitHub Container Registry (recommended):**

```
docker run -d \
  -e DISCORD_TOKEN=your_token_here \
  -v ./data:/app \
  --name anilist \
  --restart unless-stopped \
  ghcr.io/kunihir0/anilist:main
```
*(Note: A volume mount `/app` is recommended to persist `anilist.db` across container restarts).*

**Build locally:**

```
docker build -t anilist .

docker run -d \
  -e DISCORD_TOKEN=your_token_here \
  -e GUILD_ID=your_guild_id_here \
  --name anilist \
  --restart unless-stopped \
  anilist
```

`GUILD_ID` registers commands instantly to one guild — useful for development. Without it, commands register globally and take up to one hour to propagate.

### Railway

1. Create a new service and select **Deploy a Docker Image**
2. Enter the image path: `ghcr.io/kunihir0/anilist:main`
3. Add environment variables in the **Variables** tab:
   ```
   DISCORD_TOKEN=your_token_here
   GUILD_ID=your_guild_id_here
   ```
4. Disable the public URL under **Settings → Networking** — the bot makes outbound connections only

## Dependencies

- [poise](https://github.com/serenity-rs/poise) — slash command framework
- [serenity](https://github.com/serenity-rs/serenity) — Discord API
- [reqwest](https://github.com/seanmonstar/reqwest) — HTTP client
- [tokio](https://tokio.rs) — async runtime
- [serde](https://serde.rs) — JSON deserialization
- [chrono](https://github.com/chronotope/chrono) — date and time
- [futures](https://github.com/rust-lang/futures-rs) — async pagination stream
- [sqlx](https://github.com/launchbadge/sqlx) — async SQL database toolkit
- [moka](https://github.com/moka-rs/moka) — high performance caching