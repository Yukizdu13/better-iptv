use rusqlite::{Connection, Result, params};
use super::models::*;

// ========== Playlist Operations ==========

pub fn create_playlist(conn: &Connection, playlist: &Playlist) -> Result<i64> {
    conn.execute(
        "INSERT INTO playlists (name, url, file_path, auto_refresh)
         VALUES (?1, ?2, ?3, ?4)",
        params![playlist.name, playlist.url, playlist.file_path, playlist.auto_refresh],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, file_path, last_updated, auto_refresh, created_at
         FROM playlists
         ORDER BY created_at DESC"
    )?;

    let playlists = stmt.query_map([], |row| {
        Ok(Playlist {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            file_path: row.get(3)?,
            last_updated: row.get(4)?,
            auto_refresh: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(playlists)
}

pub fn delete_playlist(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

// ========== Channel Operations ==========

pub fn create_channel(conn: &Connection, channel: &Channel) -> Result<i64> {
    conn.execute(
        "INSERT INTO channels (playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            channel.playlist_id,
            channel.name,
            channel.url,
            channel.logo,
            channel.group_name,
            channel.epg_id,
            channel.tvg_name,
            channel.content_type,
            channel.sort_order
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_channels(conn: &Connection, playlist_id: Option<i64>) -> Result<Vec<Channel>> {
    let query = if let Some(pid) = playlist_id {
        format!(
            "SELECT id, playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, is_favorite, sort_order, created_at
             FROM channels
             WHERE playlist_id = {}
             ORDER BY sort_order, name",
            pid
        )
    } else {
        "SELECT id, playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, is_favorite, sort_order, created_at
         FROM channels
         ORDER BY sort_order, name".to_string()
    };

    let mut stmt = conn.prepare(&query)?;
    let channels = stmt.query_map([], |row| {
        Ok(Channel {
            id: row.get(0)?,
            playlist_id: row.get(1)?,
            name: row.get(2)?,
            url: row.get(3)?,
            logo: row.get(4)?,
            group_name: row.get(5)?,
            epg_id: row.get(6)?,
            tvg_name: row.get(7)?,
            content_type: row.get(8)?,
            is_favorite: row.get(9)?,
            sort_order: row.get(10)?,
            created_at: row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(channels)
}

pub fn search_channels(conn: &Connection, query: &str) -> Result<Vec<Channel>> {
    let search_pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, is_favorite, sort_order, created_at
         FROM channels
         WHERE name LIKE ?1 OR group_name LIKE ?1
         ORDER BY is_favorite DESC, name
         LIMIT 100"
    )?;

    let channels = stmt.query_map(params![search_pattern], |row| {
        Ok(Channel {
            id: row.get(0)?,
            playlist_id: row.get(1)?,
            name: row.get(2)?,
            url: row.get(3)?,
            logo: row.get(4)?,
            group_name: row.get(5)?,
            epg_id: row.get(6)?,
            tvg_name: row.get(7)?,
            content_type: row.get(8)?,
            is_favorite: row.get(9)?,
            sort_order: row.get(10)?,
            created_at: row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(channels)
}

pub fn toggle_favorite(conn: &Connection, channel_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE channels SET is_favorite = NOT is_favorite WHERE id = ?1",
        params![channel_id],
    )?;
    Ok(())
}

pub fn get_favorites(conn: &Connection) -> Result<Vec<Channel>> {
    let mut stmt = conn.prepare(
        "SELECT id, playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, is_favorite, sort_order, created_at
         FROM channels
         WHERE is_favorite = 1
         ORDER BY name"
    )?;

    let channels = stmt.query_map([], |row| {
        Ok(Channel {
            id: row.get(0)?,
            playlist_id: row.get(1)?,
            name: row.get(2)?,
            url: row.get(3)?,
            logo: row.get(4)?,
            group_name: row.get(5)?,
            epg_id: row.get(6)?,
            tvg_name: row.get(7)?,
            content_type: row.get(8)?,
            is_favorite: row.get(9)?,
            sort_order: row.get(10)?,
            created_at: row.get(11)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(channels)
}

// ========== Settings Operations ==========

pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;

    if let Some(row) = rows.next()? {
        Ok(Some(row.get(0)?))
    } else {
        Ok(None)
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at)
         VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        params![key, value],
    )?;
    Ok(())
}
