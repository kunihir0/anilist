-- Airing Subscriptions
CREATE TABLE IF NOT EXISTS airing_subscriptions (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    guild_id INTEGER,
    channel_id INTEGER,
    media_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    UNIQUE(user_id, media_id, channel_id)
);
