use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuildSettings {
    pub mod_role_id: Option<u64>,
    pub watch_party: Option<WatchParty>,
    pub accent_color: Option<u32>,
    #[serde(default)]
    pub server_list: Vec<ServerListEntry>,
    #[serde(default)]
    pub quiz_scores: HashMap<u64, QuizScoreInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QuizScoreInfo {
    pub score: u32,
    pub current_streak: u32,
    pub best_streak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerListEntry {
    pub id: String,
    pub media_id: u64,
    pub title: String,
    pub added_by: u64,
    pub watched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchParty {
    pub media_id: u64,
    pub title: String,
    pub channel_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserPrefs {
    pub title_language: Option<TitleLanguage>,
    #[serde(default)]
    pub compact_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, poise::ChoiceParameter, PartialEq, Eq)]
pub enum TitleLanguage {
    #[name = "Romaji"]
    Romaji,
    #[name = "English"]
    English,
    #[name = "Native"]
    Native,
}

impl std::fmt::Display for TitleLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TitleLanguage::Romaji => write!(f, "Romaji"),
            TitleLanguage::English => write!(f, "English"),
            TitleLanguage::Native => write!(f, "Native"),
        }
    }
}

impl FromStr for TitleLanguage {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Romaji" => Ok(TitleLanguage::Romaji),
            "English" => Ok(TitleLanguage::English),
            "Native" => Ok(TitleLanguage::Native),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentType {
    #[serde(rename = "daily-anime")]
    DailyAnime,
    #[serde(rename = "daily-manga")]
    DailyManga,
    #[serde(rename = "airing-update")]
    AiringUpdate,
    #[serde(rename = "trending")]
    Trending,
    #[serde(rename = "new-season")]
    NewSeason,
    #[serde(rename = "staff-birthday")]
    StaffBirthday,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::DailyAnime => write!(f, "daily-anime"),
            ContentType::DailyManga => write!(f, "daily-manga"),
            ContentType::AiringUpdate => write!(f, "airing-update"),
            ContentType::Trending => write!(f, "trending"),
            ContentType::NewSeason => write!(f, "new-season"),
            ContentType::StaffBirthday => write!(f, "staff-birthday"),
        }
    }
}

impl FromStr for ContentType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "daily-anime" => Ok(ContentType::DailyAnime),
            "daily-manga" => Ok(ContentType::DailyManga),
            "airing-update" => Ok(ContentType::AiringUpdate),
            "trending" => Ok(ContentType::Trending),
            "new-season" => Ok(ContentType::NewSeason),
            "staff-birthday" => Ok(ContentType::StaffBirthday),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub id: String,
    pub guild_id: u64,
    pub channel_id: u64,
    pub content_type: ContentType,
    pub cron_expression: String,
    pub timezone: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiringSubscription {
    pub id: String,
    pub user_id: u64,
    pub guild_id: Option<u64>,
    pub channel_id: Option<u64>,
    pub media_id: u64,
    pub title: String,
}

pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub async fn new(db_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let pool = SqlitePool::connect(db_url).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn get_mod_role(&self, guild_id: u64) -> Option<u64> {
        let g_id = guild_id as i64;
        let row = sqlx::query("SELECT mod_role_id FROM guild_settings WHERE guild_id = ?")
            .bind(g_id)
            .fetch_optional(&self.pool)
            .await
            .ok()??;

        let mod_role_id: Option<i64> = row.try_get("mod_role_id").ok()?;
        mod_role_id.map(|id| id as u64)
    }

    pub async fn set_mod_role(&self, guild_id: u64, role_id: u64) -> Result<(), sqlx::Error> {
        let g_id = guild_id as i64;
        let r_id = role_id as i64;
        sqlx::query(
            "INSERT INTO guild_settings (guild_id, mod_role_id) VALUES (?, ?)
             ON CONFLICT(guild_id) DO UPDATE SET mod_role_id = excluded.mod_role_id",
        )
        .bind(g_id)
        .bind(r_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_accent_color(&self, guild_id: u64, color: u32) -> Result<(), sqlx::Error> {
        let g_id = guild_id as i64;
        let c_val = color as i64;
        sqlx::query(
            "INSERT INTO guild_settings (guild_id, accent_color) VALUES (?, ?)
             ON CONFLICT(guild_id) DO UPDATE SET accent_color = excluded.accent_color",
        )
        .bind(g_id)
        .bind(c_val)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_settings(&self, guild_id: u64) -> GuildSettings {
        let g_id = guild_id as i64;
        let mut settings = GuildSettings::default();

        // Fetch main settings
        if let Ok(Some(row)) = sqlx::query("SELECT mod_role_id, accent_color, watch_party_media_id, watch_party_title, watch_party_channel_id FROM guild_settings WHERE guild_id = ?")
            .bind(g_id)
            .fetch_optional(&self.pool)
            .await
        {
            let mod_role_id: Option<i64> = row.try_get("mod_role_id").unwrap_or_default();
            let accent_color: Option<i64> = row.try_get("accent_color").unwrap_or_default();

            settings.mod_role_id = mod_role_id.map(|id| id as u64);
            settings.accent_color = accent_color.map(|c| c as u32);

            let m_id: Option<i64> = row.try_get("watch_party_media_id").unwrap_or_default();
            let title: Option<String> = row.try_get("watch_party_title").unwrap_or_default();
            let c_id: Option<i64> = row.try_get("watch_party_channel_id").unwrap_or_default();

            if let (Some(m_id), Some(title), Some(c_id)) = (m_id, title, c_id) {
                settings.watch_party = Some(WatchParty {
                    media_id: m_id as u64,
                    title,
                    channel_id: c_id as u64,
                });
            }
        }

        // Fetch server list
        if let Ok(rows) = sqlx::query(
            "SELECT id, media_id, title, added_by, watched FROM server_list WHERE guild_id = ?",
        )
        .bind(g_id)
        .fetch_all(&self.pool)
        .await
        {
            settings.server_list = rows
                .into_iter()
                .map(|r| ServerListEntry {
                    id: r.get("id"),
                    media_id: r.get::<i64, _>("media_id") as u64,
                    title: r.get("title"),
                    added_by: r.get::<i64, _>("added_by") as u64,
                    watched: r.get("watched"),
                })
                .collect();
        }

        // Fetch quiz scores
        if let Ok(rows) = sqlx::query("SELECT user_id, score, current_streak, best_streak FROM quiz_scores WHERE guild_id = ?")
            .bind(g_id)
            .fetch_all(&self.pool)
            .await
        {
            settings.quiz_scores = rows
                .into_iter()
                .map(|r| {
                    (
                        r.get::<i64, _>("user_id") as u64,
                        QuizScoreInfo {
                            score: r.get::<i64, _>("score") as u32,
                            current_streak: r.try_get::<i64, _>("current_streak").unwrap_or(0) as u32,
                            best_streak: r.try_get::<i64, _>("best_streak").unwrap_or(0) as u32,
                        },
                    )
                })
                .collect();
        }

        settings
    }

    pub async fn set_watch_party(
        &self,
        guild_id: u64,
        party: WatchParty,
    ) -> Result<(), sqlx::Error> {
        let g_id = guild_id as i64;
        let m_id = party.media_id as i64;
        let c_id = party.channel_id as i64;
        sqlx::query(
            "INSERT INTO guild_settings (guild_id, watch_party_media_id, watch_party_title, watch_party_channel_id)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(guild_id) DO UPDATE SET
                watch_party_media_id = excluded.watch_party_media_id,
                watch_party_title = excluded.watch_party_title,
                watch_party_channel_id = excluded.watch_party_channel_id"
        )
        .bind(g_id)
        .bind(m_id)
        .bind(party.title)
        .bind(c_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_to_server_list(
        &self,
        guild_id: u64,
        entry: ServerListEntry,
    ) -> Result<(), sqlx::Error> {
        let g_id = guild_id as i64;
        let m_id = entry.media_id as i64;
        let a_id = entry.added_by as i64;

        // Ensure guild_settings row exists to satisfy foreign key
        sqlx::query(
            "INSERT INTO guild_settings (guild_id) VALUES (?) ON CONFLICT(guild_id) DO NOTHING",
        )
        .bind(g_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO server_list (id, guild_id, media_id, title, added_by, watched) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(entry.id)
        .bind(g_id)
        .bind(m_id)
        .bind(entry.title)
        .bind(a_id)
        .bind(entry.watched)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_watched(&self, guild_id: u64, entry_id: &str) -> Result<bool, sqlx::Error> {
        let g_id = guild_id as i64;
        let result =
            sqlx::query("UPDATE server_list SET watched = 1 WHERE guild_id = ? AND id = ?")
                .bind(g_id)
                .bind(entry_id)
                .execute(&self.pool)
                .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn increment_quiz_score(
        &self,
        guild_id: u64,
        user_id: u64,
        streak_broken: bool,
    ) -> Result<(), sqlx::Error> {
        let g_id = guild_id as i64;
        let u_id = user_id as i64;

        // Ensure guild_settings row exists to satisfy foreign key
        sqlx::query(
            "INSERT INTO guild_settings (guild_id) VALUES (?) ON CONFLICT(guild_id) DO NOTHING",
        )
        .bind(g_id)
        .execute(&self.pool)
        .await?;

        if streak_broken {
            sqlx::query(
                "INSERT INTO quiz_scores (guild_id, user_id, score, current_streak, best_streak) VALUES (?, ?, 0, 0, 0)
                 ON CONFLICT(guild_id, user_id) DO UPDATE SET current_streak = 0",
            )
            .bind(g_id)
            .bind(u_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO quiz_scores (guild_id, user_id, score, current_streak, best_streak) VALUES (?, ?, 1, 1, 1)
                 ON CONFLICT(guild_id, user_id) DO UPDATE SET 
                 score = score + 1,
                 current_streak = current_streak + 1,
                 best_streak = MAX(best_streak, current_streak + 1)",
            )
            .bind(g_id)
            .bind(u_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn add_schedule(&self, entry: ScheduleEntry) -> Result<(), sqlx::Error> {
        let g_id = entry.guild_id as i64;
        let c_id = entry.channel_id as i64;
        let c_type = entry.content_type.to_string();

        sqlx::query(
            "INSERT INTO schedules (id, guild_id, channel_id, content_type, cron_expression, timezone, active)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(entry.id)
        .bind(g_id)
        .bind(c_id)
        .bind(c_type)
        .bind(entry.cron_expression)
        .bind(entry.timezone)
        .bind(entry.active)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_schedule(&self, guild_id: u64, id: &str) -> Result<bool, sqlx::Error> {
        let g_id = guild_id as i64;
        let result = sqlx::query("DELETE FROM schedules WHERE guild_id = ? AND id = ?")
            .bind(g_id)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn list_schedules(&self, guild_id: u64) -> Vec<ScheduleEntry> {
        let g_id = guild_id as i64;
        if let Ok(rows) = sqlx::query("SELECT id, channel_id, content_type, cron_expression, timezone, active FROM schedules WHERE guild_id = ?")
            .bind(g_id)
            .fetch_all(&self.pool)
            .await
        {
            rows.into_iter().filter_map(|r| {
                let ct_str: String = r.get("content_type");
                ContentType::from_str(&ct_str).ok().map(|ct| ScheduleEntry {
                    id: r.get("id"),
                    guild_id,
                    channel_id: r.get::<i64, _>("channel_id") as u64,
                    content_type: ct,
                    cron_expression: r.get("cron_expression"),
                    timezone: r.get("timezone"),
                    active: r.get("active"),
                })
            }).collect()
        } else {
            Vec::new()
        }
    }

    pub async fn toggle_schedule(
        &self,
        guild_id: u64,
        id: &str,
    ) -> Result<Option<bool>, sqlx::Error> {
        let g_id = guild_id as i64;
        let current = sqlx::query("SELECT active FROM schedules WHERE guild_id = ? AND id = ?")
            .bind(g_id)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = current {
            let active: bool = row.get("active");
            let new_state = !active;
            sqlx::query("UPDATE schedules SET active = ? WHERE guild_id = ? AND id = ?")
                .bind(new_state)
                .bind(g_id)
                .bind(id)
                .execute(&self.pool)
                .await?;
            Ok(Some(new_state))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_schedules(&self) -> HashMap<u64, Vec<ScheduleEntry>> {
        let mut map: HashMap<u64, Vec<ScheduleEntry>> = HashMap::new();
        if let Ok(rows) = sqlx::query("SELECT id, guild_id, channel_id, content_type, cron_expression, timezone, active FROM schedules").fetch_all(&self.pool).await {
            for r in rows {
                let ct_str: String = r.get("content_type");
                if let Ok(ct) = ContentType::from_str(&ct_str) {
                    let g_id = r.get::<i64, _>("guild_id") as u64;
                    map.entry(g_id).or_default().push(ScheduleEntry {
                        id: r.get("id"),
                        guild_id: g_id,
                        channel_id: r.get::<i64, _>("channel_id") as u64,
                        content_type: ct,
                        cron_expression: r.get("cron_expression"),
                        timezone: r.get("timezone"),
                        active: r.get("active"),
                    });
                }
            }
        }
        map
    }

    pub async fn get_user_prefs(&self, user_id: u64) -> UserPrefs {
        let u_id = user_id as i64;
        if let Ok(Some(row)) =
            sqlx::query("SELECT title_language, compact_mode FROM user_prefs WHERE user_id = ?")
                .bind(u_id)
                .fetch_optional(&self.pool)
                .await
        {
            let lang_str: Option<String> = row.try_get("title_language").unwrap_or_default();
            let compact_mode: bool = row.try_get("compact_mode").unwrap_or(false);
            UserPrefs {
                title_language: lang_str.and_then(|s| TitleLanguage::from_str(&s).ok()),
                compact_mode,
            }
        } else {
            UserPrefs::default()
        }
    }

    pub async fn set_compact_mode(
        &self,
        user_id: u64,
        compact_mode: bool,
    ) -> Result<(), sqlx::Error> {
        let u_id = user_id as i64;
        sqlx::query(
            "INSERT INTO user_prefs (user_id, compact_mode) VALUES (?, ?)
             ON CONFLICT(user_id) DO UPDATE SET compact_mode = excluded.compact_mode",
        )
        .bind(u_id)
        .bind(compact_mode)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn set_title_language(
        &self,
        user_id: u64,
        lang: TitleLanguage,
    ) -> Result<(), sqlx::Error> {
        let u_id = user_id as i64;
        let lang_str = lang.to_string();
        sqlx::query(
            "INSERT INTO user_prefs (user_id, title_language) VALUES (?, ?)
             ON CONFLICT(user_id) DO UPDATE SET title_language = excluded.title_language",
        )
        .bind(u_id)
        .bind(lang_str)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_airing_subscription(
        &self,
        sub: AiringSubscription,
    ) -> Result<(), sqlx::Error> {
        let g_id = sub.guild_id.map(|id| id as i64);
        let c_id = sub.channel_id.map(|id| id as i64);

        sqlx::query(
            "INSERT INTO airing_subscriptions (id, user_id, guild_id, channel_id, media_id, title)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&sub.id)
        .bind(sub.user_id as i64)
        .bind(g_id)
        .bind(c_id)
        .bind(sub.media_id as i64)
        .bind(&sub.title)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_airing_subscription(
        &self,
        user_id: u64,
        media_id: u64,
        channel_id: Option<u64>,
    ) -> Result<bool, sqlx::Error> {
        let u_id = user_id as i64;
        let m_id = media_id as i64;
        let c_id = channel_id.map(|id| id as i64);

        let query = if c_id.is_some() {
            "DELETE FROM airing_subscriptions WHERE user_id = ? AND media_id = ? AND channel_id = ?"
        } else {
            "DELETE FROM airing_subscriptions WHERE user_id = ? AND media_id = ? AND channel_id IS NULL"
        };

        let mut q = sqlx::query(query).bind(u_id).bind(m_id);
        if let Some(cid) = c_id {
            q = q.bind(cid);
        }

        let result = q.execute(&self.pool).await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_user_subscriptions(&self, user_id: u64) -> Vec<AiringSubscription> {
        let u_id = user_id as i64;
        if let Ok(rows) = sqlx::query("SELECT id, guild_id, channel_id, media_id, title FROM airing_subscriptions WHERE user_id = ?")
            .bind(u_id)
            .fetch_all(&self.pool)
            .await
        {
            rows.into_iter().map(|r| AiringSubscription {
                id: r.get("id"),
                user_id,
                guild_id: r.get::<Option<i64>, _>("guild_id").map(|id| id as u64),
                channel_id: r.get::<Option<i64>, _>("channel_id").map(|id| id as u64),
                media_id: r.get::<i64, _>("media_id") as u64,
                title: r.get("title"),
            }).collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_all_airing_subscriptions(&self) -> Vec<AiringSubscription> {
        if let Ok(rows) = sqlx::query(
            "SELECT id, user_id, guild_id, channel_id, media_id, title FROM airing_subscriptions",
        )
        .fetch_all(&self.pool)
        .await
        {
            rows.into_iter()
                .map(|r| AiringSubscription {
                    id: r.get("id"),
                    user_id: r.get::<i64, _>("user_id") as u64,
                    guild_id: r.get::<Option<i64>, _>("guild_id").map(|id| id as u64),
                    channel_id: r.get::<Option<i64>, _>("channel_id").map(|id| id as u64),
                    media_id: r.get::<i64, _>("media_id") as u64,
                    title: r.get("title"),
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}
