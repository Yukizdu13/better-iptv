use rusqlite::{Connection, Result, Row, params};
use std::collections::HashMap;
use super::models::*;
use crate::utils::generate_epg_id_swedish;

// ========== Channel Query Helpers ==========

/// SQL columns for channel SELECT queries (in order)
const CHANNEL_SELECT_COLUMNS: &str =
    "id, playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, is_favorite, sort_order, category_order, created_at";

/// Maps a database row to a Channel struct
fn map_channel_row(row: &Row) -> rusqlite::Result<Channel> {
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
        category_order: row.get(11)?,
        created_at: row.get(12)?,
    })
}

// ========== Playlist Operations ==========

pub fn create_playlist(conn: &Connection, playlist: &Playlist) -> Result<i64> {
    conn.execute(
        "INSERT INTO playlists (name, url, file_path, auto_refresh, xtream_username, xtream_password)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            playlist.name,
            playlist.url,
            playlist.file_path,
            playlist.auto_refresh,
            playlist.xtream_username,
            playlist.xtream_password
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_playlists(conn: &Connection) -> Result<Vec<Playlist>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, url, file_path, last_updated, auto_refresh, xtream_username, xtream_password, created_at
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
            xtream_username: row.get(6)?,
            xtream_password: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(playlists)
}

pub fn delete_playlist(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn rename_playlist(conn: &Connection, playlist_id: i64, new_name: &str) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET name = ?1 WHERE id = ?2",
        params![new_name, playlist_id],
    )?;
    Ok(())
}

// ========== Channel Operations ==========

pub fn create_channel(conn: &Connection, channel: &Channel) -> Result<i64> {
    conn.execute(
        "INSERT INTO channels (playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, sort_order, category_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            channel.playlist_id,
            channel.name,
            channel.url,
            channel.logo,
            channel.group_name,
            channel.epg_id,
            channel.tvg_name,
            channel.content_type,
            channel.sort_order,
            channel.category_order
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn create_channels_batch(conn: &Connection, channels: &[Channel]) -> Result<()> {
    let tx = conn.unchecked_transaction()?;

    for channel in channels {
        tx.execute(
            "INSERT INTO channels (playlist_id, name, url, logo, group_name, epg_id, tvg_name, content_type, sort_order, category_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                channel.playlist_id,
                channel.name,
                channel.url,
                channel.logo,
                channel.group_name,
                channel.epg_id,
                channel.tvg_name,
                channel.content_type,
                channel.sort_order,
                channel.category_order
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn get_channels(conn: &Connection, playlist_id: Option<i64>) -> Result<Vec<Channel>> {
    if let Some(pid) = playlist_id {
        let sql = format!(
            "SELECT {} FROM channels WHERE playlist_id = ?1 ORDER BY sort_order, name",
            CHANNEL_SELECT_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let channels = stmt.query_map(params![pid], map_channel_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(channels)
    } else {
        let sql = format!(
            "SELECT {} FROM channels ORDER BY sort_order, name",
            CHANNEL_SELECT_COLUMNS
        );
        let mut stmt = conn.prepare(&sql)?;
        let channels = stmt.query_map([], map_channel_row)?
            .collect::<Result<Vec<_>>>()?;
        Ok(channels)
    }
}

pub fn search_channels(conn: &Connection, query: &str) -> Result<Vec<Channel>> {
    let search_pattern = format!("%{}%", query);
    let sql = format!(
        "SELECT {} FROM channels WHERE name LIKE ?1 OR group_name LIKE ?1 ORDER BY is_favorite DESC, name LIMIT 100",
        CHANNEL_SELECT_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;

    let channels = stmt.query_map(params![search_pattern], map_channel_row)?
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
    let sql = format!(
        "SELECT {} FROM channels WHERE is_favorite = 1 ORDER BY name",
        CHANNEL_SELECT_COLUMNS
    );
    let mut stmt = conn.prepare(&sql)?;

    let channels = stmt.query_map([], map_channel_row)?
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

/// Delete a setting by key
pub fn delete_setting(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
    Ok(())
}

/// Get multiple settings in a single query for efficiency
pub fn get_multiple_settings(conn: &Connection, keys: &[&str]) -> Result<HashMap<String, String>> {
    if keys.is_empty() {
        return Ok(HashMap::new());
    }

    let placeholders = keys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("SELECT key, value FROM settings WHERE key IN ({})", placeholders);

    let mut stmt = conn.prepare(&sql)?;
    let result = stmt.query_map(rusqlite::params_from_iter(keys), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?
    .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(result)
}

// ========== EPG Operations ==========

/// Update EPG IDs for all Swedish channels based on their names
/// Uses a transaction with prepared statement for batch efficiency
pub fn update_channel_epg_ids(conn: &Connection) -> Result<usize> {
    // Get all live channels without EPG IDs
    let mut stmt = conn.prepare(
        "SELECT id, name FROM channels WHERE content_type = 'live' AND epg_id IS NULL"
    )?;
    let channels: Vec<(i64, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    drop(stmt); // Explicitly drop to release borrow

    if channels.is_empty() {
        return Ok(0);
    }

    // Batch update using transaction for ~100-1000x performance improvement
    let tx = conn.unchecked_transaction()?;
    let mut updated_count = 0;

    {
        let mut update_stmt = tx.prepare_cached(
            "UPDATE channels SET epg_id = ?1 WHERE id = ?2"
        )?;

        for (id, name) in &channels {
            if let Some(epg_id) = generate_epg_id_swedish(name) {
                update_stmt.execute(params![epg_id, id])?;
                updated_count += 1;
            }
        }
    }

    tx.commit()?;
    Ok(updated_count)
}

// ========== Category Operations ==========

/// Get all unique category/group names for a playlist, optionally filtered by content type
/// Categories are ordered by their original provider order (category_order), not alphabetically
pub fn get_channel_groups(
    conn: &Connection,
    playlist_id: i64,
    content_type: Option<&str>,
) -> Result<Vec<String>> {
    // Use MIN(category_order) to get the order from provider
    // Group by group_name to get distinct values, order by the min category_order
    if let Some(ct) = content_type {
        let sql = "SELECT group_name, MIN(category_order) as cat_order FROM channels
                   WHERE playlist_id = ?1 AND group_name IS NOT NULL AND group_name != ''
                   AND content_type = ?2
                   GROUP BY group_name
                   ORDER BY cat_order, group_name";
        let mut stmt = conn.prepare(sql)?;
        let groups = stmt.query_map(params![playlist_id, ct], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(groups)
    } else {
        let sql = "SELECT group_name, MIN(category_order) as cat_order FROM channels
                   WHERE playlist_id = ?1 AND group_name IS NOT NULL AND group_name != ''
                   GROUP BY group_name
                   ORDER BY cat_order, group_name";
        let mut stmt = conn.prepare(sql)?;
        let groups = stmt.query_map(params![playlist_id], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(groups)
    }
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::init_schema;

    /// Create an in-memory test database
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    /// Create a test playlist
    fn create_test_playlist(conn: &Connection, name: &str) -> i64 {
        let playlist = Playlist {
            id: None,
            name: name.to_string(),
            url: Some("http://example.com/playlist.m3u".to_string()),
            file_path: None,
            last_updated: None,
            auto_refresh: false,
            xtream_username: None,
            xtream_password: None,
            created_at: None,
        };
        create_playlist(conn, &playlist).unwrap()
    }

    /// Create a test channel
    fn create_test_channel(conn: &Connection, playlist_id: i64, name: &str) -> i64 {
        let channel = Channel {
            id: None,
            playlist_id,
            name: name.to_string(),
            url: "http://example.com/stream.m3u8".to_string(),
            logo: None,
            group_name: Some("Test Group".to_string()),
            epg_id: None,
            tvg_name: None,
            content_type: "live".to_string(),
            is_favorite: false,
            sort_order: 0,
            category_order: 0,
            created_at: None,
        };
        create_channel(conn, &channel).unwrap()
    }

    // ========== Playlist Tests ==========

    #[test]
    fn test_create_playlist_returns_id() {
        let conn = setup_test_db();
        let id = create_test_playlist(&conn, "Test Playlist");
        assert!(id > 0);
    }

    #[test]
    fn test_create_multiple_playlists() {
        let conn = setup_test_db();
        let id1 = create_test_playlist(&conn, "Playlist 1");
        let id2 = create_test_playlist(&conn, "Playlist 2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_get_playlists_returns_all() {
        let conn = setup_test_db();
        create_test_playlist(&conn, "Playlist 1");
        create_test_playlist(&conn, "Playlist 2");

        let playlists = get_playlists(&conn).unwrap();
        assert_eq!(playlists.len(), 2);
    }

    #[test]
    fn test_delete_playlist() {
        let conn = setup_test_db();
        let id = create_test_playlist(&conn, "To Delete");

        delete_playlist(&conn, id).unwrap();

        let playlists = get_playlists(&conn).unwrap();
        assert!(playlists.is_empty());
    }

    #[test]
    fn test_rename_playlist() {
        let conn = setup_test_db();
        let id = create_test_playlist(&conn, "Old Name");

        rename_playlist(&conn, id, "New Name").unwrap();

        let playlists = get_playlists(&conn).unwrap();
        assert_eq!(playlists[0].name, "New Name");
    }

    // ========== Channel Tests ==========

    #[test]
    fn test_create_channel() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");
        let channel_id = create_test_channel(&conn, playlist_id, "Test Channel");

        assert!(channel_id > 0);
    }

    #[test]
    fn test_get_channels_by_playlist() {
        let conn = setup_test_db();
        let playlist1 = create_test_playlist(&conn, "Playlist 1");
        let playlist2 = create_test_playlist(&conn, "Playlist 2");

        create_test_channel(&conn, playlist1, "Channel 1");
        create_test_channel(&conn, playlist1, "Channel 2");
        create_test_channel(&conn, playlist2, "Channel 3");

        let channels1 = get_channels(&conn, Some(playlist1)).unwrap();
        let channels2 = get_channels(&conn, Some(playlist2)).unwrap();

        assert_eq!(channels1.len(), 2);
        assert_eq!(channels2.len(), 1);
    }

    #[test]
    fn test_search_channels() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");

        create_test_channel(&conn, playlist_id, "SVT1 HD");
        create_test_channel(&conn, playlist_id, "SVT2 HD");
        create_test_channel(&conn, playlist_id, "TV4 HD");

        let results = search_channels(&conn, "SVT").unwrap();
        assert_eq!(results.len(), 2);

        let results = search_channels(&conn, "TV4").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_search_channels_case_insensitive() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");
        create_test_channel(&conn, playlist_id, "SVT1 HD");

        let results = search_channels(&conn, "svt").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_toggle_favorite() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");
        let channel_id = create_test_channel(&conn, playlist_id, "Test Channel");

        // Initially not favorite
        let channels = get_channels(&conn, Some(playlist_id)).unwrap();
        assert!(!channels[0].is_favorite);

        // Toggle to favorite
        toggle_favorite(&conn, channel_id).unwrap();
        let channels = get_channels(&conn, Some(playlist_id)).unwrap();
        assert!(channels[0].is_favorite);

        // Toggle back
        toggle_favorite(&conn, channel_id).unwrap();
        let channels = get_channels(&conn, Some(playlist_id)).unwrap();
        assert!(!channels[0].is_favorite);
    }

    #[test]
    fn test_get_favorites() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");
        let channel1 = create_test_channel(&conn, playlist_id, "Channel 1");
        let _channel2 = create_test_channel(&conn, playlist_id, "Channel 2");

        toggle_favorite(&conn, channel1).unwrap();

        let favorites = get_favorites(&conn).unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].name, "Channel 1");
    }

    #[test]
    fn test_batch_create_channels() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");

        let channels: Vec<Channel> = (0..100)
            .map(|i| Channel {
                id: None,
                playlist_id,
                name: format!("Channel {}", i),
                url: format!("http://example.com/stream{}.m3u8", i),
                logo: None,
                group_name: Some("Batch Test".to_string()),
                epg_id: None,
                tvg_name: None,
                content_type: "live".to_string(),
                is_favorite: false,
                sort_order: i,
                category_order: 0,
                created_at: None,
            })
            .collect();

        create_channels_batch(&conn, &channels).unwrap();

        let stored = get_channels(&conn, Some(playlist_id)).unwrap();
        assert_eq!(stored.len(), 100);
    }

    // ========== Settings Tests ==========

    #[test]
    fn test_get_set_setting() {
        let conn = setup_test_db();

        // Initially empty
        let value = get_setting(&conn, "theme").unwrap();
        assert!(value.is_none());

        // Set and get
        set_setting(&conn, "theme", "dark").unwrap();
        let value = get_setting(&conn, "theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn test_update_setting() {
        let conn = setup_test_db();

        set_setting(&conn, "theme", "light").unwrap();
        set_setting(&conn, "theme", "dark").unwrap();

        let value = get_setting(&conn, "theme").unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[test]
    fn test_get_multiple_settings() {
        let conn = setup_test_db();

        set_setting(&conn, "theme", "dark").unwrap();
        set_setting(&conn, "volume", "80").unwrap();
        set_setting(&conn, "language", "sv").unwrap();

        let settings = get_multiple_settings(&conn, &["theme", "volume"]).unwrap();

        assert_eq!(settings.len(), 2);
        assert_eq!(settings.get("theme"), Some(&"dark".to_string()));
        assert_eq!(settings.get("volume"), Some(&"80".to_string()));
    }

    #[test]
    fn test_get_multiple_settings_empty() {
        let conn = setup_test_db();

        let settings = get_multiple_settings(&conn, &[]).unwrap();
        assert!(settings.is_empty());
    }

    // ========== Category Tests ==========

    #[test]
    fn test_get_channel_groups() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");

        // Create channels with different groups - note category_order to test ordering
        let groups = [("Sweden", 0), ("Norway", 1), ("Denmark", 2)];
        for (i, (group, cat_order)) in groups.iter().enumerate() {
            let channel = Channel {
                id: None,
                playlist_id,
                name: format!("Channel {}", i),
                url: "http://example.com/stream.m3u8".to_string(),
                logo: None,
                group_name: Some(group.to_string()),
                epg_id: None,
                tvg_name: None,
                content_type: "live".to_string(),
                is_favorite: false,
                sort_order: i as i32,
                category_order: *cat_order,
                created_at: None,
            };
            create_channel(&conn, &channel).unwrap();
        }

        let result = get_channel_groups(&conn, playlist_id, None).unwrap();
        assert_eq!(result.len(), 3);
        // Check that order is preserved (Sweden first, then Norway, then Denmark)
        assert_eq!(result[0], "Sweden");
        assert_eq!(result[1], "Norway");
        assert_eq!(result[2], "Denmark");
    }

    #[test]
    fn test_get_channel_groups_by_content_type() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");

        // Create live channel
        let live_channel = Channel {
            id: None,
            playlist_id,
            name: "Live Channel".to_string(),
            url: "http://example.com/live.m3u8".to_string(),
            logo: None,
            group_name: Some("Live Group".to_string()),
            epg_id: None,
            tvg_name: None,
            content_type: "live".to_string(),
            is_favorite: false,
            sort_order: 0,
            category_order: 0,
            created_at: None,
        };
        create_channel(&conn, &live_channel).unwrap();

        // Create VOD channel
        let vod_channel = Channel {
            id: None,
            playlist_id,
            name: "VOD Channel".to_string(),
            url: "http://example.com/vod.m3u8".to_string(),
            logo: None,
            group_name: Some("VOD Group".to_string()),
            epg_id: None,
            tvg_name: None,
            content_type: "vod".to_string(),
            is_favorite: false,
            sort_order: 1,
            category_order: 0,
            created_at: None,
        };
        create_channel(&conn, &vod_channel).unwrap();

        // Filter by live
        let live_groups = get_channel_groups(&conn, playlist_id, Some("live")).unwrap();
        assert_eq!(live_groups.len(), 1);
        assert_eq!(live_groups[0], "Live Group");

        // Filter by vod
        let vod_groups = get_channel_groups(&conn, playlist_id, Some("vod")).unwrap();
        assert_eq!(vod_groups.len(), 1);
        assert_eq!(vod_groups[0], "VOD Group");
    }

    // ========== Cascade Delete Tests ==========

    #[test]
    fn test_delete_playlist_cascades_to_channels() {
        let conn = setup_test_db();
        let playlist_id = create_test_playlist(&conn, "Test Playlist");
        create_test_channel(&conn, playlist_id, "Channel 1");
        create_test_channel(&conn, playlist_id, "Channel 2");

        // Verify channels exist
        let channels = get_channels(&conn, Some(playlist_id)).unwrap();
        assert_eq!(channels.len(), 2);

        // Delete playlist
        delete_playlist(&conn, playlist_id).unwrap();

        // Verify channels are deleted
        let all_channels = get_channels(&conn, None).unwrap();
        assert!(all_channels.is_empty());
    }
}
