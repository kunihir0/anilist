-- Guild Settings
CREATE TABLE IF NOT EXISTS guild_settings (
    guild_id INTEGER PRIMARY KEY,
    mod_role_id INTEGER,
    accent_color INTEGER,
    -- watch party fields
    watch_party_media_id INTEGER,
    watch_party_title TEXT,
    watch_party_channel_id INTEGER
);

-- Server Anime List
CREATE TABLE IF NOT EXISTS server_list (
    id TEXT PRIMARY KEY,
    guild_id INTEGER NOT NULL,
    media_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    added_by INTEGER NOT NULL,
    watched BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY(guild_id) REFERENCES guild_settings(guild_id) ON DELETE CASCADE
);

-- Quiz Scores
CREATE TABLE IF NOT EXISTS quiz_scores (
    guild_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, user_id),
    FOREIGN KEY(guild_id) REFERENCES guild_settings(guild_id) ON DELETE CASCADE
);

-- Schedules
CREATE TABLE IF NOT EXISTS schedules (
    id TEXT PRIMARY KEY,
    guild_id INTEGER NOT NULL,
    channel_id INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    cron_expression TEXT NOT NULL,
    timezone TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT 1
);

-- User Preferences
CREATE TABLE IF NOT EXISTS user_prefs (
    user_id INTEGER PRIMARY KEY,
    title_language TEXT
);
