use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuildSettings {
    pub mod_role_id: Option<u64>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersistentData {
    pub settings: HashMap<u64, GuildSettings>,
    pub schedules: HashMap<u64, Vec<ScheduleEntry>>,
}

pub struct Store {
    path: PathBuf,
    data: RwLock<PersistentData>,
}

impl Store {
    pub async fn new(path: PathBuf) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let data = if path.exists() {
            let content = tokio::fs::read_to_string(&path).await?;
            serde_json::from_str(&content)?
        } else {
            PersistentData::default()
        };

        Ok(Self {
            path,
            data: RwLock::new(data),
        })
    }

    pub async fn get_mod_role(&self, guild_id: u64) -> Option<u64> {
        let data = self.data.read().await;
        data.settings.get(&guild_id).and_then(|s| s.mod_role_id)
    }

    pub async fn set_mod_role(&self, guild_id: u64, role_id: u64) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut data = self.data.write().await;
            let entry = data.settings.entry(guild_id).or_insert_with(GuildSettings::default);
            entry.mod_role_id = Some(role_id);
        }
        self.save().await
    }

    pub async fn get_settings(&self, guild_id: u64) -> GuildSettings {
        let data = self.data.read().await;
        data.settings.get(&guild_id).cloned().unwrap_or_default()
    }

    pub async fn add_schedule(&self, entry: ScheduleEntry) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut data = self.data.write().await;
            let guild_schedules = data.schedules.entry(entry.guild_id).or_insert_with(Vec::new);
            guild_schedules.push(entry);
        }
        self.save().await
    }

    pub async fn remove_schedule(&self, guild_id: u64, id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let removed = {
            let mut data = self.data.write().await;
            if let Some(guild_schedules) = data.schedules.get_mut(&guild_id) {
                let initial_len = guild_schedules.len();
                guild_schedules.retain(|s| s.id != id);
                initial_len != guild_schedules.len()
            } else {
                false
            }
        };
        if removed {
            self.save().await?;
        }
        Ok(removed)
    }

    pub async fn list_schedules(&self, guild_id: u64) -> Vec<ScheduleEntry> {
        let data = self.data.read().await;
        data.schedules.get(&guild_id).cloned().unwrap_or_default()
    }

    pub async fn toggle_schedule(&self, guild_id: u64, id: &str) -> Result<Option<bool>, Box<dyn std::error::Error + Send + Sync>> {
        let new_state = {
            let mut data = self.data.write().await;
            if let Some(guild_schedules) = data.schedules.get_mut(&guild_id) {
                if let Some(entry) = guild_schedules.iter_mut().find(|s| s.id == id) {
                    entry.active = !entry.active;
                    Some(entry.active)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if new_state.is_some() {
            self.save().await?;
        }
        Ok(new_state)
    }

    pub async fn get_all_schedules(&self) -> HashMap<u64, Vec<ScheduleEntry>> {
        let data = self.data.read().await;
        data.schedules.clone()
    }

    async fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = self.data.read().await;
        let content = serde_json::to_string_pretty(&*data)?;
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }
}
