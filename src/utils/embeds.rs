use poise::serenity_prelude::{self as serenity, CreateEmbed};

use crate::models::responses::{
    AniListUser, Character, Media, MediaRecommendationInfo, Staff, StaffBirthday, Studio,
    UserFavourites, MediaListCollection,
};
use crate::store::{TitleLanguage, ServerListEntry};
use std::collections::HashMap;

const ANILIST_BLUE: u32 = 0x02a9ff;
const PURPLE:       u32 = 0x9b59b6;
const TEAL:         u32 = 0x1abc9c;

fn get_color(guild_color: Option<u32>, default: u32) -> serenity::Colour {
    serenity::Colour::new(guild_color.unwrap_or(default))
}

// ─── Media (anime / manga / upcoming / random / filter) ────────────────────────

pub fn media_embed(media: &Media, media_type: &str, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let description = media
        .description.as_deref()
        .map(clean_html)
        .map(|d| truncate(&d, 300))
        .unwrap_or_else(|| "No description available.".to_string());

    let genres = if media.genres.is_empty() {
        "N/A".to_string()
    } else {
        media.genres.join(", ")
    };

    let score  = media.average_score.map(|s| format!("{s}/100")).unwrap_or_else(|| "N/A".to_string());
    let status = media.status.as_deref().unwrap_or("Unknown");
    let format = media.format.as_deref().unwrap_or("Unknown");
    let title  = media.title.get_title(lang);

    let mut embed = CreateEmbed::new()
        .title(title)
        .url(&media.site_url)
        .description(&description)
        .colour(get_color(guild_color, ANILIST_BLUE))
        .footer(serenity::CreateEmbedFooter::new(format!("{media_type} • AniList ID {}", media.id)))
        .field("Format", format, true)
        .field("Status", status, true)
        .field("Score",  &score, true)
        .field("Genres", &genres, false);

    if let Some(date) = &media.start_date {
        embed = embed.field("Start Date", date.display(), true);
    }
    if let Some(eps) = media.episodes {
        embed = embed.field("Episodes", eps.to_string(), true);
    }
    if let (Some(season), Some(year)) = (&media.season, media.season_year) {
        embed = embed.field("Season", format!("{season} {year}"), true);
    }
    if let Some(ch) = media.chapters {
        embed = embed.field("Chapters", ch.to_string(), true);
    }
    if let Some(vol) = media.volumes {
        embed = embed.field("Volumes", vol.to_string(), true);
    }
    if let Some(url) = &media.cover_image.as_ref().and_then(|c| c.large.as_ref()) {
        embed = embed.thumbnail(url.to_string());
    }

    embed
}

// ─── Airing embed (single show, for airing list page) ────────────────────────

pub fn airing_page_embed(shows: &[Media], page: usize, total_pages: usize, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title("Currently Airing")
        .colour(get_color(guild_color, TEAL))
        .footer(serenity::CreateEmbedFooter::new(format!("Page {page} of {total_pages}")));

    for show in shows {
        let title = show.title.get_title(lang.clone());
        let value = match &show.next_airing_episode {
            Some(ep) => format!("Ep {} — in {}", ep.episode, ep.countdown()),
            None => "Airing".to_string(),
        };
        embed = embed.field(title, &value, true);
    }

    embed
}

// ─── Upcoming embed ───────────────────────────────────────────────────────────

pub fn upcoming_page_embed(
    shows: &[Media],
    season: &str,
    year: i32,
    page: usize,
    total_pages: usize,
    lang: Option<TitleLanguage>,
    guild_color: Option<u32>,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("Upcoming — {season} {year}"))
        .colour(get_color(guild_color, PURPLE))
        .footer(serenity::CreateEmbedFooter::new(format!("Page {page} of {total_pages}")));

    for show in shows {
        let title = show.title.get_title(lang.clone());
        let start = show.start_date.as_ref().map(|d| d.display()).unwrap_or_else(|| "TBA".to_string());
        let score = show.average_score.map(|s| format!(" • {s}/100")).unwrap_or_default();
        embed = embed.field(title, format!("Starts {start}{score}"), true);
    }

    embed
}

// ─── Character embed ──────────────────────────────────────────────────────────

pub fn character_embed(character: &Character, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let description = character
        .description.as_deref()
        .map(clean_html)
        .map(|d| truncate(&d, 400))
        .unwrap_or_else(|| "No description available.".to_string());

    let name   = character.name.preferred();
    let native = character.name.native.as_deref().unwrap_or("");

    let appearances: String = character
        .media
        .edges
        .iter()
        .map(|e| {
            let title = e.node.title.get_title(lang.clone());
            let kind  = e.node.media_type.as_deref().unwrap_or("?");
            let mut s = format!("[{title}]({}) `{kind}`", e.node.site_url);
            if let Some(va) = e.voice_actors.first() {
                if let Some(va_name) = &va.name.full {
                    s.push_str(&format!(" (VA: [{va_name}]({}))", va.site_url));
                }
            }
            s
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut embed = CreateEmbed::new()
        .title(if native.is_empty() { name.to_string() } else { format!("{name}  ({native})") })
        .url(&character.site_url)
        .description(&description)
        .colour(get_color(guild_color, PURPLE))
        .footer(serenity::CreateEmbedFooter::new(format!("AniList Character ID {}", character.id)));

    if !appearances.is_empty() {
        embed = embed.field("Appearances", &appearances, false);
    }
    if let Some(url) = &character.image.large {
        embed = embed.thumbnail(url);
    }

    embed
}

// ─── Studio embed ─────────────────────────────────────────────────────────────

pub fn studio_embed(studio: &Studio, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let kind = if studio.is_animation_studio {
        "Animation Studio"
    } else {
        "Studio"
    };

    let works: String = studio
        .media
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let title  = n.title.get_title(lang.clone());
            let year   = n.season_year.map(|y| format!(" ({y})")).unwrap_or_default();
            let score  = n.average_score.map(|s| format!(" • {s}/100")).unwrap_or_default();
            let format = n.format.as_deref().unwrap_or("?");
            format!("{}. [{title}]({}) `{format}`{year}{score}", i + 1, n.site_url)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut embed = CreateEmbed::new()
        .title(&studio.name)
        .url(&studio.site_url)
        .colour(get_color(guild_color, TEAL))
        .footer(serenity::CreateEmbedFooter::new(format!("{kind} • AniList ID {}", studio.id)));

    if !works.is_empty() {
        embed = embed.field("Notable Works", &works, false);
    }

    embed
}

// ─── Staff embed ─────────────────────────────────────────────────────────────

pub fn staff_embed(staff: &Staff, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let description = staff
        .description.as_deref()
        .map(clean_html)
        .map(|d| truncate(&d, 400))
        .unwrap_or_else(|| "No description available.".to_string());

    let name   = staff.name.preferred();
    let native = staff.name.native.as_deref().unwrap_or("");
    let bday   = if staff.is_birthday { " 🎂" } else { "" };

    let works: String = staff
        .staff_media
        .nodes
        .iter()
        .map(|n| {
            let title = n.title.get_title(lang.clone());
            let kind  = n.media_type.as_deref().unwrap_or("?");
            format!("[{title}]({}) `{kind}`", n.site_url)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut embed = CreateEmbed::new()
        .title(if native.is_empty() { format!("{name}{bday}") } else { format!("{name}  ({native}){bday}") })
        .url(&staff.site_url)
        .description(&description)
        .colour(get_color(guild_color, PURPLE))
        .footer(serenity::CreateEmbedFooter::new(format!("AniList Staff ID {}", staff.id)));

    if !works.is_empty() {
        embed = embed.field("Works", &works, false);
    }
    if let Some(url) = &staff.image.large {
        embed = embed.thumbnail(url);
    }

    embed
}

// ─── Staff Birthday embed ────────────────────────────────────────────────────

pub fn staff_birthday_embed(staff_list: &[StaffBirthday], guild_color: Option<u32>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title("🎂 Today's Staff Birthdays")
        .colour(get_color(guild_color, PURPLE))
        .footer(serenity::CreateEmbedFooter::new("Powered by AniList"));

    if staff_list.is_empty() {
        embed = embed.description("No staff birthdays found for today.");
    } else {
        let list: String = staff_list.iter()
            .take(15)
            .map(|s| format!("[{}]({})", s.name.preferred(), s.site_url))
            .collect::<Vec<_>>()
            .join("\n");
        embed = embed.description(list);
    }

    embed
}

// ─── Recommendations embed ───────────────────────────────────────────────────

pub fn recommendations_embed(media: &MediaRecommendationInfo, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let title = media.title.get_title(lang.clone());
    let recs: String = media
        .recommendations
        .nodes
        .iter()
        .filter_map(|n| n.media_recommendation.as_ref())
        .map(|r| format!("[{}]({})", r.title.get_title(lang.clone()), r.site_url))
        .collect::<Vec<_>>()
        .join("\n");

    CreateEmbed::new()
        .title(format!("Recommendations for {title}"))
        .url(format!("https://anilist.co/anime/{}/recommendations", media.id))
        .description(if recs.is_empty() { "No recommendations found.".to_string() } else { recs })
        .colour(get_color(guild_color, ANILIST_BLUE))
}

// ─── Relations embed ─────────────────────────────────────────────────────────

pub fn relations_embed(media: &Media, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let title = media.title.get_title(lang.clone());
    let mut embed = CreateEmbed::new()
        .title(format!("Relations for {title}"))
        .url(&media.site_url)
        .colour(get_color(guild_color, ANILIST_BLUE));

    if let Some(relations) = &media.relations {
        let list: String = relations.edges.iter()
            .map(|e| {
                let r_type = e.relation_type.replace('_', " ");
                let r_title = e.node.title.get_title(lang.clone());
                let r_format = e.node.format.as_deref().unwrap_or("?");
                format!("`{r_type}`: [{r_title}]({}) ({r_format})", e.node.site_url)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        embed = embed.description(if list.is_empty() { "No relations found.".to_string() } else { list });
    } else {
        embed = embed.description("No relations found.");
    }

    embed
}

// ─── Media list embed (trending / genre / filter) ─────────────────────────────────────

pub fn media_list_embed(media: &[Media], title: &str, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let list: String = media
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let score = m.average_score.map(|s| format!(" • {s}/100")).unwrap_or_default();
            format!("{}. [{}]({}){}", i + 1, m.title.get_title(lang.clone()), m.site_url, score)
        })
        .collect::<Vec<_>>()
        .join("\n");

    CreateEmbed::new()
        .title(title)
        .description(if list.is_empty() { "No results found.".to_string() } else { list })
        .colour(get_color(guild_color, ANILIST_BLUE))
}

// ─── Watchlist embed ─────────────────────────────────────────────────────────

pub fn watchlist_embed(collection: &MediaListCollection, username: &str, media_type: &str, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("{}'s {} Watchlist", username, media_type))
        .colour(get_color(guild_color, ANILIST_BLUE));

    for list in &collection.lists {
        if list.entries.is_empty() { continue; }
        
        let entries: String = list.entries.iter().take(10)
            .map(|e| {
                let title = e.media.title.get_title(lang.clone());
                let score = if e.score > 0.0 { format!(" ({}/100)", e.score) } else { "".to_string() };
                format!("[{}]({}){}", title, e.media.site_url, score)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        embed = embed.field(&list.name, entries, false);
    }

    embed
}

// ─── Favourites embed ────────────────────────────────────────────────────────

pub fn favourites_embed(user: &UserFavourites, lang: Option<TitleLanguage>, guild_color: Option<u32>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("{}'s Favourites", user.name))
        .url(&user.site_url)
        .colour(get_color(guild_color, ANILIST_BLUE));

    let anime: String = user.favourites.anime.nodes.iter()
        .map(|n| format!("[{}]({})", n.title.get_title(lang.clone()), n.site_url))
        .collect::<Vec<_>>()
        .join("\n");
    if !anime.is_empty() { embed = embed.field("Anime", anime, true); }

    let manga: String = user.favourites.manga.nodes.iter()
        .map(|n| format!("[{}]({})", n.title.get_title(lang.clone()), n.site_url))
        .collect::<Vec<_>>()
        .join("\n");
    if !manga.is_empty() { embed = embed.field("Manga", manga, true); }

    let characters: String = user.favourites.characters.nodes.iter()
        .map(|n| format!("[{}]({})", n.name.preferred(), n.site_url))
        .collect::<Vec<_>>()
        .join("\n");
    if !characters.is_empty() { embed = embed.field("Characters", characters, true); }

    let studios: String = user.favourites.studios.nodes.iter()
        .map(|n| format!("[{}]({})", n.name, n.site_url))
        .collect::<Vec<_>>()
        .join("\n");
    if !studios.is_empty() { embed = embed.field("Studios", studios, true); }

    embed
}

// ─── User profile embed ───────────────────────────────────────────────────────

pub fn user_embed(user: &AniListUser, guild_color: Option<u32>) -> CreateEmbed {
    let about = user
        .about.as_deref()
        .map(clean_html)
        .map(|a| truncate(&a, 200))
        .unwrap_or_else(|| "No bio set.".to_string());

    let anime = &user.statistics.anime;
    let manga = &user.statistics.manga;
    let days  = anime.minutes_watched / 1440;
    let hours = (anime.minutes_watched % 1440) / 60;

    let mut embed = CreateEmbed::new()
        .title(&user.name)
        .url(&user.site_url)
        .description(&about)
        .colour(get_color(guild_color, ANILIST_BLUE))
        .footer(serenity::CreateEmbedFooter::new(format!("AniList User ID {}", user.id)))
        .field("Anime Watched",    anime.count.to_string(),                   true)
        .field("Episodes Watched", anime.episodes_watched.to_string(),        true)
        .field("Watch Time",       format!("{days}d {hours}h"),               true)
        .field("Anime Mean Score", format!("{:.1}/100", anime.mean_score),    true)
        .field("Manga Read",       manga.count.to_string(),                   true)
        .field("Chapters Read",    manga.chapters_read.to_string(),           true)
        .field("Manga Mean Score", format!("{:.1}/100", manga.mean_score),    true);

    if !user.statistics.anime.genres.is_empty() {
        let favorite_genres = user.statistics.anime.genres.iter()
            .take(5)
            .map(|g| format!("{} ({})", g.genre, g.count))
            .collect::<Vec<_>>()
            .join(", ");
        embed = embed.field("Top Genres", favorite_genres, false);
    }

    if let Some(url) = &user.avatar.large {
        embed = embed.thumbnail(url);
    }

    embed
}

// ─── Compare embed ────────────────────────────────────────────────────────────

pub fn compare_embed(u1: &AniListUser, u2: &AniListUser, guild_color: Option<u32>) -> CreateEmbed {
    fn user_stats(u: &AniListUser) -> String {
        let days  = u.statistics.anime.minutes_watched / 1440;
        let hours = (u.statistics.anime.minutes_watched % 1440) / 60;
        format!(
            "Anime watched: {}\nEpisodes: {}\nWatch time: {}d {}h\nAnime score: {:.1}\n\nManga read: {}\nChapters: {}\nManga score: {:.1}",
            u.statistics.anime.count,
            u.statistics.anime.episodes_watched,
            days, hours,
            u.statistics.anime.mean_score,
            u.statistics.manga.count,
            u.statistics.manga.chapters_read,
            u.statistics.manga.mean_score,
        )
    }

    CreateEmbed::new()
        .title(format!("{} vs {}", u1.name, u2.name))
        .colour(get_color(guild_color, ANILIST_BLUE))
        .footer(serenity::CreateEmbedFooter::new("AniList profile comparison"))
        .field(format!("[{}]({})", u1.name, u1.site_url), user_stats(u1), true)
        .field(format!("[{}]({})", u2.name, u2.site_url), user_stats(u2), true)
}

// ─── Server List embed ────────────────────────────────────────────────────────

pub fn server_list_embed(entries: &[ServerListEntry], guild_color: Option<u32>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title("Server Anime List")
        .colour(get_color(guild_color, TEAL));

    if entries.is_empty() {
        embed = embed.description("The server list is empty. Add something with `/serverlist add`!");
    } else {
        let list: String = entries.iter()
            .map(|e| {
                let status = if e.watched { "✅" } else { "⏳" };
                format!("`{}` {} **{}** (added by <@{}>)", e.id, status, e.title, e.added_by)
            })
            .collect::<Vec<_>>()
            .join("\n");
        embed = embed.description(list);
    }

    embed
}

// ─── Leaderboard embed ────────────────────────────────────────────────────────

pub fn leaderboard_embed(scores: &HashMap<u64, u32>, guild_color: Option<u32>) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title("Quiz Leaderboard")
        .colour(get_color(guild_color, 0xFFA500));

    if scores.is_empty() {
        embed = embed.description("No scores yet. Play with `/quiz`!");
    } else {
        let mut sorted: Vec<_> = scores.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));

        let list: String = sorted.iter().enumerate()
            .take(10)
            .map(|(i, (id, score))| format!("{}. <@{}> — **{}** wins", i + 1, id, score))
            .collect::<Vec<_>>()
            .join("\n");
        embed = embed.description(list);
    }

    embed
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn clean_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}
