//! Contenu synchronisable d'un compte et sa construction depuis la base locale.
//!
//! Ne sont PAS synchronisés : `channels` et `epg_programs` (réimportables depuis le fournisseur),
//! l'historique `watch_history` (non alimenté par l'application), les réglages parentaux (le
//! hash du PIN reste local) et `active_profile_id` (identifiant local).
//!
//! Favoris, progression et presets ne pointent pas sur `channel_id` (propre à chaque base) mais
//! sur une clé stable : `(uuid du profil, clé de chaîne)`.

use crate::db::mutations::extract_stream_id_from_url;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const SNAPSHOT_VERSION: u32 = 1;

/// Horodatage « jamais modifié » (format SQLite `CURRENT_TIMESTAMP`, comparable en texte).
pub const EPOCH: &str = "1970-01-01 00:00:00";
/// Horodatage des favoris existants avant la migration : plus récent que `EPOCH`, pour qu'un
/// favori ancien s'applique sur un appareil qui ne l'a jamais touché.
pub const LEGACY: &str = "1970-01-01 00:00:01";

/// Réglages synchronisés (liste blanche). Tout le reste reste local.
pub const SYNCED_SETTINGS: &[&str] = &[
    "theme",
    "language",
    "volume",
    "video_quality",
    "epg_url",
    "audio_language",
    "subtitle_language",
    "playlist_user_agent_mode",
    "playlist_user_agent_custom",
    "default_advanced_filters",
    "credits_skip_seconds",
];

pub fn is_synced_setting(key: &str) -> bool {
    SYNCED_SETTINGS.contains(&key)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Snapshot {
    pub version: u32,
    pub created_at: String,
    pub profiles: Vec<ProfileRec>,
    pub favorites: Vec<FavoriteRec>,
    pub series_progress: Vec<ProgressRec>,
    pub presets: Vec<PresetRec>,
    pub settings: Vec<SettingRec>,
    pub tombstones: Vec<TombstoneRec>,
}

/// Profil (playlist). Contient des secrets Xtream : `Debug` les masque.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileRec {
    pub uuid: String,
    pub name: String,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub auto_refresh: bool,
    pub xtream_username: Option<String>,
    pub xtream_password: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: String,
}

impl std::fmt::Debug for ProfileRec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileRec")
            .field("uuid", &self.uuid)
            .field("name", &self.name)
            .field("updated_at", &self.updated_at)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FavoriteRec {
    pub playlist_uuid: String,
    pub channel_key: String,
    pub is_favorite: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgressRec {
    pub playlist_uuid: String,
    pub channel_key: String,
    pub season: String,
    pub episode_id: String,
    pub episode_num: i32,
    pub title: String,
    pub position_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetRec {
    pub uuid: String,
    pub playlist_uuid: String,
    pub name: String,
    pub series_keys: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SettingRec {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TombstoneRec {
    /// `"profile"` ou `"preset"`
    pub kind: String,
    pub uuid: String,
    pub deleted_at: String,
}

/// Clé stable d'une chaîne entre appareils : identifiant de flux Xtream si l'URL en contient un,
/// sinon `(nom, groupe)` (même repli que `merge_channels` pour les M3U).
pub fn channel_key(content_type: &str, url: &str, name: &str, group: Option<&str>) -> String {
    match extract_stream_id_from_url(url) {
        Some(id) => format!("x:{}:{}", content_type, id),
        None => format!("n:{}|{}", name, group.unwrap_or("")),
    }
}

/// `channel_key -> ids locaux` pour un profil.
pub fn keys_to_ids(conn: &Connection, playlist_id: i64) -> Result<HashMap<String, Vec<i64>>> {
    let mut stmt = conn.prepare(
        "SELECT id, content_type, url, name, group_name FROM channels WHERE playlist_id = ?1",
    )?;
    let mut map: HashMap<String, Vec<i64>> = HashMap::new();
    let rows = stmt.query_map(params![playlist_id], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<String>>(4)?,
        ))
    })?;
    for row in rows {
        let (id, ct, url, name, group) = row?;
        map.entry(channel_key(&ct, &url, &name, group.as_deref()))
            .or_default()
            .push(id);
    }
    Ok(map)
}

/// `id local -> channel_key` pour un profil.
fn ids_to_keys(conn: &Connection, playlist_id: i64) -> Result<HashMap<i64, String>> {
    let mut map = HashMap::new();
    for (key, ids) in keys_to_ids(conn, playlist_id)? {
        for id in ids {
            map.insert(id, key.clone());
        }
    }
    Ok(map)
}

/// Construit le snapshot de la base locale.
pub fn build_snapshot(conn: &Connection) -> Result<Snapshot> {
    let mut snap = Snapshot {
        version: SNAPSHOT_VERSION,
        created_at: now_utc(conn)?,
        ..Default::default()
    };

    // Profils
    let mut uuid_by_id: HashMap<i64, String> = HashMap::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, uuid, name, url, file_path, auto_refresh, xtream_username, xtream_password,
                    created_at, COALESCE(updated_at, ?1)
             FROM playlists WHERE uuid IS NOT NULL ORDER BY id",
        )?;
        let rows = stmt.query_map(params![EPOCH], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                ProfileRec {
                    uuid: r.get(1)?,
                    name: r.get(2)?,
                    url: r.get(3)?,
                    file_path: r.get(4)?,
                    auto_refresh: r.get(5)?,
                    xtream_username: r.get(6)?,
                    xtream_password: r.get(7)?,
                    created_at: r.get(8)?,
                    updated_at: r.get(9)?,
                },
            ))
        })?;
        for row in rows {
            let (id, rec) = row?;
            uuid_by_id.insert(id, rec.uuid.clone());
            snap.profiles.push(rec);
        }
    }

    // Favoris : chaînes actuellement favorites ou dont le favori a été retiré depuis la migration
    {
        let mut stmt = conn.prepare(
            "SELECT playlist_id, content_type, url, name, group_name, is_favorite,
                    COALESCE(favorite_updated_at, ?1)
             FROM channels WHERE is_favorite = 1 OR favorite_updated_at IS NOT NULL",
        )?;
        let rows = stmt.query_map(params![LEGACY], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, bool>(5)?,
                r.get::<_, String>(6)?,
            ))
        })?;
        // Dédoublonnage par clé : l'entrée la plus récente l'emporte
        let mut best: HashMap<(String, String), FavoriteRec> = HashMap::new();
        for row in rows {
            let (pid, ct, url, name, group, fav, ts) = row?;
            let Some(uuid) = uuid_by_id.get(&pid) else { continue };
            let key = channel_key(&ct, &url, &name, group.as_deref());
            let rec = FavoriteRec {
                playlist_uuid: uuid.clone(),
                channel_key: key.clone(),
                is_favorite: fav,
                updated_at: ts,
            };
            let slot = best.entry((uuid.clone(), key));
            slot.and_modify(|cur| {
                if rec.updated_at > cur.updated_at
                    || (rec.updated_at == cur.updated_at && rec.is_favorite)
                {
                    *cur = rec.clone();
                }
            })
            .or_insert(rec);
        }
        let mut favs: Vec<FavoriteRec> = best.into_values().collect();
        favs.sort_by(|a, b| (&a.playlist_uuid, &a.channel_key).cmp(&(&b.playlist_uuid, &b.channel_key)));
        snap.favorites = favs;
    }

    // Progression des séries
    {
        let mut stmt = conn.prepare(
            "SELECT c.playlist_id, c.content_type, c.url, c.name, c.group_name,
                    s.season, s.episode_id, s.episode_num, s.title,
                    s.position_seconds, s.duration_seconds, COALESCE(s.updated_at, ?1)
             FROM series_last_episode s JOIN channels c ON c.id = s.channel_id",
        )?;
        let rows = stmt.query_map(params![EPOCH], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
                ProgressRec {
                    playlist_uuid: String::new(),
                    channel_key: String::new(),
                    season: r.get(5)?,
                    episode_id: r.get(6)?,
                    episode_num: r.get(7)?,
                    title: r.get(8)?,
                    position_seconds: r.get(9)?,
                    duration_seconds: r.get(10)?,
                    updated_at: r.get(11)?,
                },
            ))
        })?;
        let mut best: HashMap<(String, String), ProgressRec> = HashMap::new();
        for row in rows {
            let (pid, ct, url, name, group, mut rec) = row?;
            let Some(uuid) = uuid_by_id.get(&pid) else { continue };
            rec.playlist_uuid = uuid.clone();
            rec.channel_key = channel_key(&ct, &url, &name, group.as_deref());
            let k = (rec.playlist_uuid.clone(), rec.channel_key.clone());
            match best.get(&k) {
                Some(cur) if cur.updated_at >= rec.updated_at => {}
                _ => {
                    best.insert(k, rec);
                }
            }
        }
        let mut list: Vec<ProgressRec> = best.into_values().collect();
        list.sort_by(|a, b| (&a.playlist_uuid, &a.channel_key).cmp(&(&b.playlist_uuid, &b.channel_key)));
        snap.series_progress = list;
    }

    // Presets de play aléatoire
    {
        let mut stmt = conn.prepare(
            "SELECT uuid, playlist_id, name, series_channel_ids, COALESCE(updated_at, ?1)
             FROM random_play_presets WHERE uuid IS NOT NULL ORDER BY id",
        )?;
        let rows: Vec<(String, i64, String, String, String)> = stmt
            .query_map(params![EPOCH], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?
            .collect::<Result<_>>()?;
        let mut key_maps: HashMap<i64, HashMap<i64, String>> = HashMap::new();
        for (uuid, pid, name, ids_json, ts) in rows {
            let Some(pl_uuid) = uuid_by_id.get(&pid) else { continue };
            let map = match key_maps.entry(pid) {
                std::collections::hash_map::Entry::Occupied(e) => e.into_mut(),
                std::collections::hash_map::Entry::Vacant(e) => e.insert(ids_to_keys(conn, pid)?),
            };
            let ids: Vec<i64> = serde_json::from_str(&ids_json).unwrap_or_default();
            // Les séries qui n'existent plus dans la liste sont ignorées
            let series_keys = ids.iter().filter_map(|id| map.get(id).cloned()).collect();
            snap.presets.push(PresetRec {
                uuid,
                playlist_uuid: pl_uuid.clone(),
                name,
                series_keys,
                updated_at: ts,
            });
        }
    }

    // Réglages (liste blanche)
    {
        let mut stmt =
            conn.prepare("SELECT key, value, COALESCE(updated_at, ?1) FROM settings ORDER BY key")?;
        let rows = stmt.query_map(params![EPOCH], |r| {
            Ok(SettingRec {
                key: r.get(0)?,
                value: r.get(1)?,
                updated_at: r.get(2)?,
            })
        })?;
        for row in rows {
            let rec = row?;
            if is_synced_setting(&rec.key) {
                snap.settings.push(rec);
            }
        }
    }

    // Suppressions
    {
        let mut stmt = conn.prepare("SELECT kind, uuid, deleted_at FROM sync_tombstones ORDER BY kind, uuid")?;
        let rows = stmt.query_map([], |r| {
            Ok(TombstoneRec {
                kind: r.get(0)?,
                uuid: r.get(1)?,
                deleted_at: r.get(2)?,
            })
        })?;
        for row in rows {
            snap.tombstones.push(row?);
        }
    }

    Ok(snap)
}

fn now_utc(conn: &Connection) -> Result<String> {
    conn.query_row("SELECT CURRENT_TIMESTAMP", [], |r| r.get(0))
}
