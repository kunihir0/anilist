use reqwest::Client;

// ─── Shared application state ────────────────────────────────────────────────
//
// This struct is initialised once in main.rs and then injected into every
// command invocation via `ctx.data()`.  Storing the reqwest::Client here means
// we reuse the same connection pool for every AniList request rather than
// opening a new TCP connection each time.

pub struct Data {
    /// Long-lived async HTTP client for all AniList GraphQL calls.
    pub http_client: Client,
}

// ─── Poise type aliases ───────────────────────────────────────────────────────
//
// As recommended by the Poise docs, we define these once so that every command
// file can simply write `Context<'_>` / `Error` instead of the verbose
// fully-qualified forms.

/// Catch-all error type: any `Error + Send + Sync` can be propagated with `?`.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Poise context carrying our `Data` and `Error` types.
pub type Context<'a> = poise::Context<'a, Data, Error>;
