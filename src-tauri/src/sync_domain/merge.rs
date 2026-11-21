//! Fusion d'un snapshot dans la base locale.
//!
//! Règles (identiques pour l'import manuel et la future synchro automatique) :
//! - profils / presets : identifiés par `uuid`, le plus récent (`updated_at`) gagne ;
//! - suppressions : un marqueur (`sync_tombstones`) plus récent que la dernière modification
//!   supprime l'élément ; une modification plus récente que le marqueur le ressuscite ;
//! - favoris : le plus récent gagne (un favori retiré est un enregistrement comme un autre) ;
//! - progression des séries : l'enregistrement le plus récent gagne ;
//! - réglages : liste blanche, le plus récent gagne.
//!
//! Les favoris / progressions / presets dont la chaîne n'est pas encore dans la base locale
//! (liste non importée sur cet appareil) sont mis en attente dans `sync_pending` et appliqués
//! après l'import des chaînes du profil (`apply_pending_for_playlist`).

use super::snapshot::*;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct MergeReport {
    pub profiles_added: usize,
    pub profiles_updated: usize,
    pub profiles_deleted: usize,
    pub favorites_applied: usize,
    pub progress_applied: usize,
    pub presets_applied: usize,
    pub settings_applied: usize,
    /// Éléments en attente de l'import des chaînes de leur profil
    pub pending: usize,
}

/// Résultat de l'application d'un enregistrement : `resolved` = la chaîne existe localement
/// (sinon l'enregistrement part en attente), `changed` = la base a réellement été modifiée.
#[derive(Debug, Clone, Copy)]
struct Applied {
    resolved: bool,
    changed: bool,
}

const KIND_FAV: &str = "fav";
const KIND_PROGRESS: &str = "progress";
const KIND_PRESET: &str = "preset";

pub fn merge_snapshot(conn: &Connection, snap: &Snapshot) -> Result<MergeReport> {
    let tx = conn.unchecked_transaction()?;
    let mut report = MergeReport::default();

    // 1. Marqueurs de suppression : on garde le plus récent par (kind, uuid)
    for t in &snap.tombstones {
        if t.kind != "profile" && t.kind != "preset" {
            continue;
        }
        tx.execute(
            "INSERT INTO sync_tombstones (kind, uuid, deleted_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(kind, uuid) DO UPDATE SET deleted_at = excluded.deleted_at
             WHERE excluded.deleted_at > sync_tombstones.deleted_at",
            params![t.kind, t.uuid, t.deleted_at],
        )?;
    }

    // 2. Profils
    for p in &snap.profiles {
        if tombstone_wins(&tx, "profile", &p.uuid, &p.updated_at)? {
            continue;
        }
        let local: Option<(i64, String)> = tx
            .query_row(
                "SELECT id, COALESCE(updated_at, ?2) FROM playlists WHERE uuid = ?1",
                params![p.uuid, EPOCH],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match local {
            None => {
                tx.execute(
                    "INSERT INTO playlists
                        (uuid, name, url, file_path, auto_refresh, xtream_username, xtream_password,
                         created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, COALESCE(?8, CURRENT_TIMESTAMP), ?9)",
                    params![
                        p.uuid,
                        p.name,
                        p.url,
                        p.file_path,
                        p.auto_refresh,
                        p.xtream_username,
                        p.xtream_password,
                        p.created_at,
                        p.updated_at
                    ],
                )?;
                clear_tombstone(&tx, "profile", &p.uuid)?;
                report.profiles_added += 1;
            }
            Some((id, local_ts)) if p.updated_at > local_ts => {
                tx.execute(
                    "UPDATE playlists SET name=?1, url=?2, file_path=?3, auto_refresh=?4,
                            xtream_username=?5, xtream_password=?6, updated_at=?7
                     WHERE id=?8",
                    params![
                        p.name,
                        p.url,
                        p.file_path,
                        p.auto_refresh,
                        p.xtream_username,
                        p.xtream_password,
                        p.updated_at,
                        id
                    ],
                )?;
                report.profiles_updated += 1;
            }
            Some(_) => {}
        }
    }

    // 3. Profils supprimés ailleurs (marqueur plus récent que la dernière modification locale)
    {
        let mut stmt = tx.prepare(
            "SELECT p.id, p.uuid FROM playlists p
             JOIN sync_tombstones t ON t.kind = 'profile' AND t.uuid = p.uuid
             WHERE t.deleted_at >= COALESCE(p.updated_at, ?1)",
        )?;
        let doomed: Vec<(i64, String)> = stmt
            .query_map(params![EPOCH], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_>>()?;
        drop(stmt);
        for (id, uuid) in doomed {
            delete_profile_data(&tx, id, &uuid)?;
            report.profiles_deleted += 1;
        }
    }

    // 4. Contenu rattaché aux profils
    let profile_ids: HashMap<String, i64> = {
        let mut stmt = tx.prepare("SELECT uuid, id FROM playlists WHERE uuid IS NOT NULL")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        rows.collect::<Result<_>>()?
    };
    let mut key_cache: HashMap<i64, HashMap<String, Vec<i64>>> = HashMap::new();

    for f in &snap.favorites {
        let Some(&pid) = profile_ids.get(&f.playlist_uuid) else { continue };
        let keys = cached_keys(&tx, &mut key_cache, pid)?;
        let r = apply_favorite(&tx, keys, f)?;
        if r.changed {
            report.favorites_applied += 1;
        }
        if !r.resolved
            && store_pending(&tx, KIND_FAV, &f.playlist_uuid, &f.channel_key, f, &f.updated_at)?
        {
            report.pending += 1;
        }
    }

    for s in &snap.series_progress {
        let Some(&pid) = profile_ids.get(&s.playlist_uuid) else { continue };
        let keys = cached_keys(&tx, &mut key_cache, pid)?;
        let r = apply_progress(&tx, keys, s)?;
        if r.changed {
            report.progress_applied += 1;
        }
        if !r.resolved
            && store_pending(&tx, KIND_PROGRESS, &s.playlist_uuid, &s.channel_key, s, &s.updated_at)?
        {
            report.pending += 1;
        }
    }

    for p in &snap.presets {
        if tombstone_wins(&tx, "preset", &p.uuid, &p.updated_at)? {
            continue;
        }
        let Some(&pid) = profile_ids.get(&p.playlist_uuid) else { continue };
        let keys = cached_keys(&tx, &mut key_cache, pid)?;
        let (applied, complete) = apply_preset(&tx, pid, keys, p)?;
        if applied {
            report.presets_applied += 1;
        }
        if !complete && store_pending(&tx, KIND_PRESET, &p.playlist_uuid, &p.uuid, p, &p.updated_at)? {
            report.pending += 1;
        }
    }

    // 5. Presets supprimés ailleurs
    tx.execute(
        "DELETE FROM random_play_presets WHERE uuid IN (
            SELECT r.uuid FROM random_play_presets r
            JOIN sync_tombstones t ON t.kind = 'preset' AND t.uuid = r.uuid
            WHERE t.deleted_at >= COALESCE(r.updated_at, ?1))",
        params![EPOCH],
    )?;

    // 6. Réglages (liste blanche)
    for s in &snap.settings {
        if !is_synced_setting(&s.key) {
            continue;
        }
        let local: Option<String> = tx
            .query_row(
                "SELECT COALESCE(updated_at, ?2) FROM settings WHERE key = ?1",
                params![s.key, EPOCH],
                |r| r.get(0),
            )
            .optional()?;
        if local.is_none_or(|ts| s.updated_at > ts) {
            tx.execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                params![s.key, s.value, s.updated_at],
            )?;
            report.settings_applied += 1;
        }
    }

    tx.commit()?;

    // Si le profil actif a été supprimé, on en désigne un autre
    if report.profiles_deleted > 0 {
        repair_active_profile(conn)?;
    }
    Ok(report)
}

/// Applique les éléments en attente d'un profil dont les chaînes viennent d'être importées.
/// Coût nul tant que `sync_pending` est vide (cas courant).
pub fn apply_pending_for_playlist(conn: &Connection, playlist_id: i64) -> Result<usize> {
    let has_any: bool = conn.query_row("SELECT EXISTS(SELECT 1 FROM sync_pending)", [], |r| r.get(0))?;
    if !has_any {
        return Ok(0);
    }
    let Some(uuid): Option<String> = conn
        .query_row("SELECT uuid FROM playlists WHERE id = ?1", params![playlist_id], |r| r.get(0))
        .optional()?
        .flatten()
    else {
        return Ok(0);
    };

    let rows: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare(
            "SELECT kind, item_key, payload FROM sync_pending WHERE playlist_uuid = ?1",
        )?;
        let rows = stmt.query_map(params![uuid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        rows.collect::<Result<_>>()?
    };
    if rows.is_empty() {
        return Ok(0);
    }

    let tx = conn.unchecked_transaction()?;
    let keys = super::snapshot::keys_to_ids(&tx, playlist_id)?;
    let mut applied = 0;
    for (kind, item_key, payload) in rows {
        let done = match kind.as_str() {
            KIND_FAV => match serde_json::from_str::<FavoriteRec>(&payload) {
                Ok(rec) => {
                    let r = apply_favorite(&tx, &keys, &rec)?;
                    applied += usize::from(r.changed);
                    r.resolved
                }
                Err(_) => true, // payload illisible : on l'écarte
            },
            KIND_PROGRESS => match serde_json::from_str::<ProgressRec>(&payload) {
                Ok(rec) => {
                    let r = apply_progress(&tx, &keys, &rec)?;
                    applied += usize::from(r.changed);
                    r.resolved
                }
                Err(_) => true,
            },
            KIND_PRESET => match serde_json::from_str::<PresetRec>(&payload) {
                Ok(rec) => {
                    if tombstone_wins(&tx, "preset", &rec.uuid, &rec.updated_at)? {
                        true
                    } else {
                        let (ok, complete) = apply_preset(&tx, playlist_id, &keys, &rec)?;
                        if ok {
                            applied += 1;
                        }
                        complete
                    }
                }
                Err(_) => true,
            },
            _ => true,
        };
        if done {
            tx.execute(
                "DELETE FROM sync_pending WHERE kind=?1 AND playlist_uuid=?2 AND item_key=?3",
                params![kind, uuid, item_key],
            )?;
        }
    }
    tx.commit()?;
    Ok(applied)
}

// ---------------------------------------------------------------------------------------------

fn cached_keys<'a>(
    conn: &Connection,
    cache: &'a mut HashMap<i64, HashMap<String, Vec<i64>>>,
    playlist_id: i64,
) -> Result<&'a HashMap<String, Vec<i64>>> {
    use std::collections::hash_map::Entry;
    let keys = match cache.entry(playlist_id) {
        Entry::Occupied(e) => e.into_mut(),
        Entry::Vacant(e) => e.insert(keys_to_ids(conn, playlist_id)?),
    };
    Ok(keys)
}

/// `true` si un marqueur de suppression local est au moins aussi récent que `updated_at`.
fn tombstone_wins(conn: &Connection, kind: &str, uuid: &str, updated_at: &str) -> Result<bool> {
    let ts: Option<String> = conn
        .query_row(
            "SELECT deleted_at FROM sync_tombstones WHERE kind=?1 AND uuid=?2",
            params![kind, uuid],
            |r| r.get(0),
        )
        .optional()?;
    Ok(ts.is_some_and(|t| t.as_str() >= updated_at))
}

fn clear_tombstone(conn: &Connection, kind: &str, uuid: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM sync_tombstones WHERE kind=?1 AND uuid=?2",
        params![kind, uuid],
    )?;
    Ok(())
}

fn apply_favorite(
    conn: &Connection,
    keys: &HashMap<String, Vec<i64>>,
    f: &FavoriteRec,
) -> Result<Applied> {
    let Some(ids) = keys.get(&f.channel_key) else {
        return Ok(Applied { resolved: false, changed: false });
    };
    let mut changed = false;
    for id in ids {
        let local_ts: String = conn.query_row(
            "SELECT COALESCE(favorite_updated_at, ?2) FROM channels WHERE id = ?1",
            params![id, EPOCH],
            |r| r.get(0),
        )?;
        if f.updated_at > local_ts {
            conn.execute(
                "UPDATE channels SET is_favorite = ?1, favorite_updated_at = ?2 WHERE id = ?3",
                params![f.is_favorite, f.updated_at, id],
            )?;
            changed = true;
        }
    }
    Ok(Applied { resolved: true, changed })
}

fn apply_progress(
    conn: &Connection,
    keys: &HashMap<String, Vec<i64>>,
    s: &ProgressRec,
) -> Result<Applied> {
    let Some(ids) = keys.get(&s.channel_key) else {
        return Ok(Applied { resolved: false, changed: false });
    };
    let mut changed = false;
    for id in ids {
        let local_ts: Option<String> = conn
            .query_row(
                "SELECT COALESCE(updated_at, ?2) FROM series_last_episode WHERE channel_id = ?1",
                params![id, EPOCH],
                |r| r.get(0),
            )
            .optional()?;
        if local_ts.is_none_or(|ts| s.updated_at > ts) {
            conn.execute(
                "INSERT INTO series_last_episode
                    (channel_id, season, episode_id, episode_num, title, position_seconds,
                     duration_seconds, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(channel_id) DO UPDATE SET
                    season = excluded.season, episode_id = excluded.episode_id,
                    episode_num = excluded.episode_num, title = excluded.title,
                    position_seconds = excluded.position_seconds,
                    duration_seconds = excluded.duration_seconds,
                    updated_at = excluded.updated_at",
                params![
                    id,
                    s.season,
                    s.episode_id,
                    s.episode_num,
                    s.title,
                    s.position_seconds,
                    s.duration_seconds,
                    s.updated_at
                ],
            )?;
            changed = true;
        }
    }
    Ok(Applied { resolved: true, changed })
}

/// Retourne `(appliqué, complet)` : `complet` est faux si certaines séries ne sont pas encore
/// dans la base locale (le preset est alors recalculé après l'import des chaînes).
fn apply_preset(
    conn: &Connection,
    playlist_id: i64,
    keys: &HashMap<String, Vec<i64>>,
    p: &PresetRec,
) -> Result<(bool, bool)> {
    let mut ids: Vec<i64> = Vec::new();
    let mut complete = true;
    for k in &p.series_keys {
        match keys.get(k) {
            Some(found) => {
                if let Some(first) = found.first() {
                    if !ids.contains(first) {
                        ids.push(*first);
                    }
                }
            }
            None => complete = false,
        }
    }
    let ids_json = serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string());

    let local: Option<String> = conn
        .query_row(
            "SELECT COALESCE(updated_at, ?2) FROM random_play_presets WHERE uuid = ?1",
            params![p.uuid, EPOCH],
            |r| r.get(0),
        )
        .optional()?;
    let applied = match local {
        None => {
            conn.execute(
                "INSERT INTO random_play_presets (uuid, playlist_id, name, series_channel_ids, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![p.uuid, playlist_id, p.name, ids_json, p.updated_at],
            )?;
            clear_tombstone(conn, "preset", &p.uuid)?;
            true
        }
        // `>=` : au rattrapage des chaînes manquantes, le preset a déjà la date du snapshot
        Some(ts) if p.updated_at >= ts && (p.updated_at > ts || !complete || ids_differ(conn, &p.uuid, &ids)?) => {
            conn.execute(
                "UPDATE random_play_presets SET name=?1, series_channel_ids=?2, updated_at=?3 WHERE uuid=?4",
                params![p.name, ids_json, p.updated_at, p.uuid],
            )?;
            true
        }
        Some(_) => false,
    };
    Ok((applied, complete))
}

fn ids_differ(conn: &Connection, uuid: &str, ids: &[i64]) -> Result<bool> {
    let cur: String = conn.query_row(
        "SELECT series_channel_ids FROM random_play_presets WHERE uuid = ?1",
        params![uuid],
        |r| r.get(0),
    )?;
    let cur: Vec<i64> = serde_json::from_str(&cur).unwrap_or_default();
    Ok(cur != ids)
}

/// Met l'enregistrement en attente s'il est plus récent que celui déjà en attente.
/// Retourne `true` si une ligne a été écrite.
fn store_pending<T: Serialize>(
    conn: &Connection,
    kind: &str,
    playlist_uuid: &str,
    item_key: &str,
    rec: &T,
    updated_at: &str,
) -> Result<bool> {
    let payload = serde_json::to_string(rec).unwrap_or_default();
    let n = conn.execute(
        "INSERT INTO sync_pending (kind, playlist_uuid, item_key, payload, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(kind, playlist_uuid, item_key) DO UPDATE SET
            payload = excluded.payload, updated_at = excluded.updated_at
         WHERE excluded.updated_at > sync_pending.updated_at",
        params![kind, playlist_uuid, item_key, payload, updated_at],
    )?;
    Ok(n > 0)
}

/// Suppression complète d'un profil (sans dépendre de `PRAGMA foreign_keys`).
fn delete_profile_data(conn: &Connection, id: i64, uuid: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM series_last_episode WHERE channel_id IN (SELECT id FROM channels WHERE playlist_id = ?1)",
        params![id],
    )?;
    conn.execute(
        "DELETE FROM watch_history WHERE channel_id IN (SELECT id FROM channels WHERE playlist_id = ?1)",
        params![id],
    )?;
    conn.execute("DELETE FROM random_play_presets WHERE playlist_id = ?1", params![id])?;
    conn.execute("DELETE FROM channels WHERE playlist_id = ?1", params![id])?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    conn.execute("DELETE FROM sync_pending WHERE playlist_uuid = ?1", params![uuid])?;
    Ok(())
}

fn repair_active_profile(conn: &Connection) -> Result<()> {
    let active = crate::db::queries::get_setting(conn, "active_profile_id")?;
    if let Some(id) = active.and_then(|s| s.parse::<i64>().ok()) {
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM playlists WHERE id = ?1)",
            params![id],
            |r| r.get(0),
        )?;
        if !exists {
            crate::db::mutations::delete_setting(conn, "active_profile_id")?;
        }
    }
    crate::db::schema::ensure_active_profile(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::{Channel, Playlist};
    use crate::db::schema::init_schema;
    use crate::db::{mutations, queries};
    use crate::sync_domain::crypto::{self, KdfParams};

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    fn playlist(conn: &Connection, name: &str) -> i64 {
        mutations::create_playlist(
            conn,
            &Playlist {
                id: None,
                name: name.into(),
                url: Some("http://exemple.test".into()),
                file_path: None,
                last_updated: None,
                auto_refresh: false,
                xtream_username: Some("user".into()),
                xtream_password: Some("pass-secret".into()),
                created_at: None,
            },
        )
        .unwrap()
    }

    fn channel(pid: i64, name: &str, stream_id: i64, ct: &str) -> Channel {
        Channel {
            id: None,
            playlist_id: pid,
            name: name.into(),
            url: format!("http://exemple.test/{}/user/pass/{}.m3u8", ct, stream_id),
            logo: None,
            group_name: Some("G".into()),
            epg_id: None,
            tvg_name: None,
            content_type: ct.into(),
            is_favorite: false,
            sort_order: 0,
            category_order: 0,
            rating: None,
            created_at: None,
        }
    }

    fn chan_id(conn: &Connection, pid: i64, name: &str) -> i64 {
        conn.query_row(
            "SELECT id FROM channels WHERE playlist_id=?1 AND name=?2",
            params![pid, name],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn is_fav(conn: &Connection, id: i64) -> bool {
        conn.query_row("SELECT is_favorite FROM channels WHERE id=?1", params![id], |r| r.get(0))
            .unwrap()
    }

    fn set_fav_at(conn: &Connection, id: i64, fav: bool, ts: &str) {
        conn.execute(
            "UPDATE channels SET is_favorite=?1, favorite_updated_at=?2 WHERE id=?3",
            params![fav, ts, id],
        )
        .unwrap();
    }

    /// Passe par le fichier chiffré, comme l'export/import réels.
    fn transfer(from: &Connection, to: &Connection) -> MergeReport {
        let snap = build_snapshot(from).unwrap();
        let json = serde_json::to_vec(&snap).unwrap();
        let enc = crypto::encrypt(&json, "phrase de test assez longue 123", &KdfParams::TEST).unwrap();
        let dec = crypto::decrypt(&enc, "phrase de test assez longue 123").unwrap();
        let back: Snapshot = serde_json::from_slice(&dec).unwrap();
        assert_eq!(snap, back);
        merge_snapshot(to, &back).unwrap()
    }

    fn uuid_of(conn: &Connection, id: i64) -> String {
        conn.query_row("SELECT uuid FROM playlists WHERE id=?1", params![id], |r| r.get(0)).unwrap()
    }

    #[test]
    fn migration_backfills_old_database() {
        let conn = Connection::open_in_memory().unwrap();
        // Base d'avant #20 : aucune colonne uuid / updated_at / favorite_updated_at
        conn.execute_batch(
            "CREATE TABLE playlists (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, url TEXT,
                file_path TEXT, last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP, auto_refresh BOOLEAN DEFAULT 0,
                xtream_username TEXT, xtream_password TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);
             INSERT INTO playlists (name) VALUES ('ancien-1'), ('ancien-2');
             CREATE TABLE random_play_presets (id INTEGER PRIMARY KEY AUTOINCREMENT, playlist_id INTEGER NOT NULL,
                name TEXT NOT NULL, series_channel_ids TEXT NOT NULL, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);
             INSERT INTO random_play_presets (playlist_id, name, series_channel_ids) VALUES (1, 'p', '[]');",
        )
        .unwrap();
        init_schema(&conn).unwrap();
        init_schema(&conn).unwrap(); // idempotent

        let uuids: Vec<String> = conn
            .prepare("SELECT uuid FROM playlists ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_>>()
            .unwrap();
        assert_eq!(uuids.len(), 2);
        assert!(uuids.iter().all(|u| u.len() == 32));
        assert_ne!(uuids[0], uuids[1]);
        let null_ts: i64 = conn
            .query_row("SELECT COUNT(*) FROM playlists WHERE updated_at IS NULL", [], |r| r.get(0))
            .unwrap();
        assert_eq!(null_ts, 0);
        let preset_uuid: Option<String> = conn
            .query_row("SELECT uuid FROM random_play_presets", [], |r| r.get(0))
            .unwrap();
        assert!(preset_uuid.is_some());

        // Une nouvelle insertion reçoit uuid et updated_at via le trigger
        let id = playlist(&conn, "nouveau");
        let (u, ts): (Option<String>, Option<String>) = conn
            .query_row("SELECT uuid, updated_at FROM playlists WHERE id=?1", params![id], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert!(u.is_some() && ts.is_some());
    }

    #[test]
    fn debug_never_prints_credentials() {
        let a = db();
        playlist(&a, "p");
        let snap = build_snapshot(&a).unwrap();
        let dbg = format!("{:?}", snap);
        assert!(!dbg.contains("pass-secret"));
        assert!(!dbg.contains("user"));
    }

    #[test]
    fn full_transfer_to_empty_device_with_pending_then_channels_import() {
        let a = db();
        let pid = playlist(&a, "brad");
        mutations::create_channels_batch(
            &a,
            &[
                channel(pid, "TF1", 11, "live"),
                channel(pid, "Big Mouth", 77, "series"),
                channel(pid, "Film", 99, "vod"),
            ],
        )
        .unwrap();
        mutations::toggle_favorite(&a, chan_id(&a, pid, "TF1")).unwrap();
        let series_id = chan_id(&a, pid, "Big Mouth");
        mutations::set_last_episode(
            &a,
            series_id,
            &mutations::LastEpisodeUpdate {
                season: "2",
                episode_id: "555",
                episode_num: 4,
                title: "Ep 4",
                position_seconds: Some(123.5),
                duration_seconds: Some(1500.0),
            },
        )
        .unwrap();
        mutations::create_random_play_preset(&a, pid, "mon preset", &[series_id]).unwrap();
        mutations::set_setting(&a, "theme", "dark").unwrap();
        mutations::set_setting(&a, "parental_pin_hash", "SECRET-HASH").unwrap();
        mutations::set_setting(&a, "active_profile_id", &pid.to_string()).unwrap();

        // Appareil B vide : profil créé, tout le reste en attente (pas de chaînes)
        let b = db();
        let rep = transfer(&a, &b);
        assert_eq!(rep.profiles_added, 1);
        assert_eq!(rep.settings_applied, 1, "seul 'theme' est synchronisé");
        assert_eq!(rep.pending, 3, "favori + progression + preset en attente");
        assert_eq!(queries::get_setting(&b, "parental_pin_hash").unwrap(), None);
        assert_eq!(queries::get_setting(&b, "active_profile_id").unwrap(), None);
        assert_eq!(queries::get_setting(&b, "theme").unwrap().as_deref(), Some("dark"));
        let b_pid: i64 = b.query_row("SELECT id FROM playlists", [], |r| r.get(0)).unwrap();
        let (u, pw): (Option<String>, Option<String>) = b
            .query_row("SELECT xtream_username, xtream_password FROM playlists", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!((u.as_deref(), pw.as_deref()), (Some("user"), Some("pass-secret")));
        assert_eq!(uuid_of(&b, b_pid), uuid_of(&a, pid));

        // L'utilisateur importe la liste sur B : les ids locaux diffèrent, les données se retrouvent
        mutations::create_channels_batch(
            &b,
            &[
                channel(b_pid, "Big Mouth", 77, "series"),
                channel(b_pid, "Film", 99, "vod"),
                channel(b_pid, "TF1", 11, "live"),
            ],
        )
        .unwrap();
        assert!(is_fav(&b, chan_id(&b, b_pid, "TF1")));
        let b_series = chan_id(&b, b_pid, "Big Mouth");
        assert_ne!(b_series, series_id, "ids locaux différents entre appareils");
        let ep = queries::get_last_episode(&b, b_series).unwrap().unwrap();
        assert_eq!((ep.episode_num, ep.position_seconds), (4, Some(123.5)));
        let presets = queries::get_random_play_presets(&b, b_pid).unwrap();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].series_channel_ids, vec![b_series]);
        let pending: i64 = b.query_row("SELECT COUNT(*) FROM sync_pending", [], |r| r.get(0)).unwrap();
        assert_eq!(pending, 0, "plus rien en attente");
    }

    #[test]
    fn favorites_last_write_wins_and_unfavorite_propagates() {
        let a = db();
        let b = db();
        let pa = playlist(&a, "p");
        mutations::create_channels_batch(&a, &[channel(pa, "TF1", 11, "live"), channel(pa, "M6", 12, "live")]).unwrap();
        transfer(&a, &b);
        let pb: i64 = b.query_row("SELECT id FROM playlists", [], |r| r.get(0)).unwrap();
        mutations::create_channels_batch(&b, &[channel(pb, "TF1", 11, "live"), channel(pb, "M6", 12, "live")]).unwrap();

        let (a_tf1, b_tf1) = (chan_id(&a, pa, "TF1"), chan_id(&b, pb, "TF1"));
        let (a_m6, b_m6) = (chan_id(&a, pa, "M6"), chan_id(&b, pb, "M6"));
        // TF1 : A le met en favori à t1, B le retire à t2 (plus récent) -> retiré partout
        set_fav_at(&a, a_tf1, true, "2026-01-01 10:00:00");
        set_fav_at(&b, b_tf1, false, "2026-01-01 11:00:00");
        // M6 : B le met en favori à t1, A le retire à t2 (plus récent) -> retiré partout
        set_fav_at(&b, b_m6, true, "2026-01-02 10:00:00");
        set_fav_at(&a, a_m6, false, "2026-01-02 11:00:00");

        transfer(&a, &b);
        transfer(&b, &a);
        for (c, id) in [(&a, a_tf1), (&b, b_tf1), (&a, a_m6), (&b, b_m6)] {
            assert!(!is_fav(c, id));
        }

        // Le plus récent peut aussi être un ajout : A remet TF1 à t3
        set_fav_at(&a, a_tf1, true, "2026-02-01 00:00:00");
        transfer(&a, &b);
        assert!(is_fav(&b, b_tf1));
        // Idempotent : refusionner ne change rien
        let again = transfer(&a, &b);
        assert_eq!(again.profiles_added + again.profiles_updated + again.profiles_deleted, 0);
        assert_eq!((again.favorites_applied, again.progress_applied, again.presets_applied, again.pending), (0, 0, 0, 0));
        assert!(is_fav(&b, b_tf1));
    }

    #[test]
    fn legacy_favorites_apply_on_untouched_device() {
        // Favori posé avant la migration (favorite_updated_at NULL) : doit passer sur l'autre appareil
        let a = db();
        let b = db();
        let pa = playlist(&a, "p");
        mutations::create_channels_batch(&a, &[channel(pa, "TF1", 11, "live")]).unwrap();
        a.execute("UPDATE channels SET is_favorite = 1", []).unwrap();
        transfer(&a, &b);
        let pb: i64 = b.query_row("SELECT id FROM playlists", [], |r| r.get(0)).unwrap();
        mutations::create_channels_batch(&b, &[channel(pb, "TF1", 11, "live")]).unwrap();
        assert!(is_fav(&b, chan_id(&b, pb, "TF1")));
    }

    #[test]
    fn series_progress_most_recent_wins() {
        let a = db();
        let b = db();
        let pa = playlist(&a, "p");
        mutations::create_channels_batch(&a, &[channel(pa, "S", 5, "series")]).unwrap();
        transfer(&a, &b);
        let pb: i64 = b.query_row("SELECT id FROM playlists", [], |r| r.get(0)).unwrap();
        mutations::create_channels_batch(&b, &[channel(pb, "S", 5, "series")]).unwrap();
        let (sa, sb) = (chan_id(&a, pa, "S"), chan_id(&b, pb, "S"));
        let upd = |c: &Connection, id: i64, n: i32, pos: f64, ts: &str| {
            mutations::set_last_episode(
                c,
                id,
                &mutations::LastEpisodeUpdate {
                    season: "1",
                    episode_id: "e",
                    episode_num: n,
                    title: "t",
                    position_seconds: Some(pos),
                    duration_seconds: Some(1000.0),
                },
            )
            .unwrap();
            c.execute("UPDATE series_last_episode SET updated_at=?1 WHERE channel_id=?2", params![ts, id]).unwrap();
        };
        upd(&a, sa, 3, 10.0, "2026-03-01 10:00:00");
        upd(&b, sb, 5, 900.0, "2026-03-02 10:00:00"); // plus récent
        transfer(&a, &b); // ne doit PAS écraser B
        assert_eq!(queries::get_last_episode(&b, sb).unwrap().unwrap().episode_num, 5);
        transfer(&b, &a); // A rattrape B
        let ep = queries::get_last_episode(&a, sa).unwrap().unwrap();
        assert_eq!((ep.episode_num, ep.position_seconds), (5, Some(900.0)));
    }

    #[test]
    fn deleted_profile_propagates_and_can_be_resurrected() {
        let a = db();
        let b = db();
        let pa = playlist(&a, "à supprimer");
        mutations::create_channels_batch(&a, &[channel(pa, "TF1", 11, "live")]).unwrap();
        transfer(&a, &b);
        let pb: i64 = b.query_row("SELECT id FROM playlists", [], |r| r.get(0)).unwrap();
        mutations::create_channels_batch(&b, &[channel(pb, "TF1", 11, "live")]).unwrap();
        mutations::set_setting(&b, "active_profile_id", &pb.to_string()).unwrap();

        // Suppression sur A, avec un marqueur postérieur à toute modification de B
        let uuid = uuid_of(&a, pa);
        mutations::delete_playlist(&a, pa).unwrap();
        a.execute("UPDATE sync_tombstones SET deleted_at = '2999-01-01 00:00:00'", []).unwrap();
        let rep = transfer(&a, &b);
        assert_eq!(rep.profiles_deleted, 1);
        let left: i64 = b.query_row("SELECT COUNT(*) FROM playlists", [], |r| r.get(0)).unwrap();
        let chans: i64 = b.query_row("SELECT COUNT(*) FROM channels", [], |r| r.get(0)).unwrap();
        assert_eq!((left, chans), (0, 0));
        assert_eq!(queries::get_setting(&b, "active_profile_id").unwrap(), None);

        // B ne doit pas recréer le profil supprimé en recevant un vieux snapshot
        let stale = Snapshot {
            version: 1,
            profiles: vec![ProfileRec {
                uuid: uuid.clone(),
                name: "vieux".into(),
                url: None,
                file_path: None,
                auto_refresh: false,
                xtream_username: None,
                xtream_password: None,
                created_at: None,
                updated_at: "2026-01-01 00:00:00".into(),
            }],
            ..Default::default()
        };
        merge_snapshot(&b, &stale).unwrap();
        let left: i64 = b.query_row("SELECT COUNT(*) FROM playlists", [], |r| r.get(0)).unwrap();
        assert_eq!(left, 0);

        // Une modification postérieure au marqueur ressuscite le profil
        let mut fresh = stale.clone();
        fresh.profiles[0].updated_at = "3000-01-01 00:00:00".into();
        let rep = merge_snapshot(&b, &fresh).unwrap();
        assert_eq!(rep.profiles_added, 1);
        let tomb: i64 = b.query_row("SELECT COUNT(*) FROM sync_tombstones", [], |r| r.get(0)).unwrap();
        assert_eq!(tomb, 0);
    }

    #[test]
    fn deleted_preset_propagates() {
        let a = db();
        let b = db();
        let pa = playlist(&a, "p");
        mutations::create_channels_batch(&a, &[channel(pa, "S", 5, "series")]).unwrap();
        let preset = mutations::create_random_play_preset(&a, pa, "x", &[chan_id(&a, pa, "S")]).unwrap();
        transfer(&a, &b);
        let n: i64 = b.query_row("SELECT COUNT(*) FROM random_play_presets", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 1);
        mutations::delete_random_play_preset(&a, preset).unwrap();
        a.execute("UPDATE sync_tombstones SET deleted_at = '2999-01-01 00:00:00'", []).unwrap();
        transfer(&a, &b);
        let n: i64 = b.query_row("SELECT COUNT(*) FROM random_play_presets", [], |r| r.get(0)).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn profile_rename_most_recent_wins() {
        let a = db();
        let b = db();
        let pa = playlist(&a, "ancien nom");
        transfer(&a, &b);
        let pb: i64 = b.query_row("SELECT id FROM playlists", [], |r| r.get(0)).unwrap();
        mutations::rename_playlist(&b, pb, "nouveau nom").unwrap();
        b.execute("UPDATE playlists SET updated_at = '2999-01-01 00:00:00'", []).unwrap();
        transfer(&a, &b); // plus ancien : ne change rien
        let name: String = b.query_row("SELECT name FROM playlists", [], |r| r.get(0)).unwrap();
        assert_eq!(name, "nouveau nom");
        let rep = transfer(&b, &a);
        assert_eq!(rep.profiles_updated, 1);
        let name: String = a.query_row("SELECT name FROM playlists WHERE id=?1", params![pa], |r| r.get(0)).unwrap();
        assert_eq!(name, "nouveau nom");
    }

    #[test]
    fn settings_whitelist_and_per_key_lww() {
        let a = db();
        let b = db();
        mutations::set_setting(&a, "theme", "dark").unwrap();
        mutations::set_setting(&a, "epg_url", "http://epg.test/a.xml").unwrap();
        mutations::set_setting(&a, "credits_skip_seconds_series_42", "30").unwrap();
        mutations::set_setting(&a, "parental_enabled", "true").unwrap();
        mutations::set_setting(&b, "theme", "light").unwrap();
        b.execute("UPDATE settings SET updated_at='2999-01-01 00:00:00' WHERE key='theme'", []).unwrap();
        transfer(&a, &b);
        assert_eq!(queries::get_setting(&b, "theme").unwrap().as_deref(), Some("light"));
        assert_eq!(queries::get_setting(&b, "epg_url").unwrap().as_deref(), Some("http://epg.test/a.xml"));
        assert_eq!(queries::get_setting(&b, "credits_skip_seconds_series_42").unwrap(), None);
        assert_eq!(queries::get_setting(&b, "parental_enabled").unwrap(), None);
    }

    #[test]
    fn forged_snapshot_cannot_inject_unlisted_settings() {
        let b = db();
        let snap = Snapshot {
            version: 1,
            settings: vec![SettingRec {
                key: "parental_pin_hash".into(),
                value: "x".into(),
                updated_at: "2999-01-01 00:00:00".into(),
            }],
            ..Default::default()
        };
        merge_snapshot(&b, &snap).unwrap();
        assert_eq!(queries::get_setting(&b, "parental_pin_hash").unwrap(), None);
    }
}
