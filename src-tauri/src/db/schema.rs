use rusqlite::{Connection, Result};

/// Initialize the database schema
pub fn init_schema(conn: &Connection) -> Result<()> {
    // Playlists table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT,
            file_path TEXT,
            last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            auto_refresh BOOLEAN DEFAULT 0,
            xtream_username TEXT,
            xtream_password TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Channels table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS channels (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            playlist_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            logo TEXT,
            group_name TEXT,
            epg_id TEXT,
            tvg_name TEXT,
            content_type TEXT DEFAULT 'live',
            is_favorite BOOLEAN DEFAULT 0,
            sort_order INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // EPG Programs table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS epg_programs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel_epg_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            start_time TIMESTAMP NOT NULL,
            end_time TIMESTAMP NOT NULL,
            category TEXT,
            icon TEXT
        )",
        [],
    )?;

    // Create index for EPG lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channel_time
         ON epg_programs(channel_epg_id, start_time, end_time)",
        [],
    )?;

    // Create index for channel search (LIKE queries on name and group_name)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channel_search
         ON channels(name, group_name)",
        [],
    )?;

    // Index for playlist filtering (frequently used in WHERE playlist_id = ?)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_playlist_id
         ON channels(playlist_id)",
        [],
    )?;

    // Index for EPG lookups by channel EPG ID
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_epg_id
         ON channels(epg_id)",
        [],
    )?;

    // Watch History table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS watch_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel_id INTEGER NOT NULL,
            watched_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            duration_seconds INTEGER DEFAULT 0,
            FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Index for watch history lookups by channel
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_watch_history_channel_id
         ON watch_history(channel_id)",
        [],
    )?;

    // Settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // EPG Sources table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS epg_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            last_fetched TIMESTAMP,
            auto_refresh BOOLEAN DEFAULT 1,
            refresh_interval_hours INTEGER DEFAULT 6,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    Ok(())
}

/// Ensure active_profile_id setting exists (migration for existing users)
pub fn ensure_active_profile(conn: &Connection) -> Result<()> {
    // Check if active_profile_id already exists
    let existing = crate::db::operations::get_setting(conn, "active_profile_id")?;

    if existing.is_none() {
        // Get first playlist (oldest by created_at)
        let playlists = crate::db::operations::get_playlists(conn)?;

        if let Some(first_playlist) = playlists.first() {
            let playlist_id = first_playlist.id.unwrap().to_string();
            crate::db::operations::set_setting(
                conn,
                "active_profile_id",
                &playlist_id
            )?;
            log::info!(
                "Migration: Set active profile to first playlist (ID: {})",
                playlist_id
            );
        }
    }

    Ok(())
}
