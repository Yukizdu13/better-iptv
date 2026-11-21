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
            category_order INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Migration: Add category_order column if it doesn't exist (for existing databases)
    let _ = conn.execute(
        "ALTER TABLE channels ADD COLUMN category_order INTEGER DEFAULT 0",
        [],
    );

    // Migration: Add rating column for VOD/series filtering
    let _ = conn.execute(
        "ALTER TABLE channels ADD COLUMN rating REAL",
        [],
    );

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

    // Composite index for channel sorting (used in get_channels ORDER BY)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_sorting
         ON channels(playlist_id, sort_order, name)",
        [],
    )?;

    // Index for content type filtering (used in category queries and content type tabs)
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_content_type
         ON channels(playlist_id, content_type)",
        [],
    )?;

    // Composite index for category queries with content type filter
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_channels_category
         ON channels(playlist_id, content_type, group_name, category_order)",
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

    // Last watched episode per series (channel_id = the series' channel row)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS series_last_episode (
            channel_id INTEGER PRIMARY KEY,
            season TEXT NOT NULL,
            episode_id TEXT NOT NULL,
            episode_num INTEGER NOT NULL,
            title TEXT NOT NULL,
            position_seconds REAL,
            duration_seconds REAL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Migration: playback position of the last episode (exact resume)
    let _ = conn.execute(
        "ALTER TABLE series_last_episode ADD COLUMN position_seconds REAL",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE series_last_episode ADD COLUMN duration_seconds REAL",
        [],
    );

    // Saved series selections for cross-series random play ("Play aléatoire multi-séries")
    conn.execute(
        "CREATE TABLE IF NOT EXISTS random_play_presets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            playlist_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            series_channel_ids TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Index for preset lookups by playlist
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_random_play_presets_playlist_id
         ON random_play_presets(playlist_id)",
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

    // ---- Compte universel (#20) : colonnes et tables de synchronisation ----
    migrate_sync_columns(conn)?;

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
    let existing = crate::db::queries::get_setting(conn, "active_profile_id")?;

    if existing.is_none() {
        // Get first playlist (oldest by created_at)
        let playlists = crate::db::queries::get_playlists(conn)?;

        if let Some(first_playlist) = playlists.first() {
            let playlist_id = first_playlist.id.unwrap().to_string();
            crate::db::mutations::set_setting(
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

/// Migration for the universal account (#20): stable identifiers (uuid), modification
/// timestamps and deletion markers, so data can be merged between devices.
///
/// Safe on existing databases: `ALTER TABLE` errors (column already there) are ignored and the
/// backfill only touches rows that have no value yet.
fn migrate_sync_columns(conn: &Connection) -> Result<()> {
    // Profiles (playlists)
    let _ = conn.execute("ALTER TABLE playlists ADD COLUMN uuid TEXT", []);
    let _ = conn.execute("ALTER TABLE playlists ADD COLUMN updated_at TIMESTAMP", []);
    conn.execute(
        "UPDATE playlists SET uuid = lower(hex(randomblob(16))) WHERE uuid IS NULL",
        [],
    )?;
    conn.execute(
        "UPDATE playlists SET updated_at = COALESCE(created_at, CURRENT_TIMESTAMP) WHERE updated_at IS NULL",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_playlists_uuid ON playlists(uuid)",
        [],
    )?;
    // New rows get their uuid / updated_at without touching every INSERT call site
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS trg_playlists_sync_defaults
         AFTER INSERT ON playlists
         WHEN NEW.uuid IS NULL OR NEW.updated_at IS NULL
         BEGIN
             UPDATE playlists
             SET uuid = COALESCE(NEW.uuid, lower(hex(randomblob(16)))),
                 updated_at = COALESCE(NEW.updated_at, CURRENT_TIMESTAMP)
             WHERE id = NEW.id;
         END",
        [],
    )?;

    // Favorites: when the favorite flag last changed (NULL = never touched since the migration)
    let _ = conn.execute("ALTER TABLE channels ADD COLUMN favorite_updated_at TIMESTAMP", []);

    // Random play presets
    let _ = conn.execute("ALTER TABLE random_play_presets ADD COLUMN uuid TEXT", []);
    conn.execute(
        "UPDATE random_play_presets SET uuid = lower(hex(randomblob(16))) WHERE uuid IS NULL",
        [],
    )?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_random_play_presets_uuid ON random_play_presets(uuid)",
        [],
    )?;
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS trg_presets_sync_defaults
         AFTER INSERT ON random_play_presets
         WHEN NEW.uuid IS NULL
         BEGIN
             UPDATE random_play_presets SET uuid = lower(hex(randomblob(16))) WHERE id = NEW.id;
         END",
        [],
    )?;

    // Deletion markers: a deleted profile / preset must disappear on the other devices too
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_tombstones (
            kind TEXT NOT NULL,
            uuid TEXT NOT NULL,
            deleted_at TIMESTAMP NOT NULL,
            PRIMARY KEY (kind, uuid)
        )",
        [],
    )?;

    // Synced data whose channel is not in the local database yet (playlist not imported on this
    // device). Re-applied after each channel import of the playlist.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_pending (
            kind TEXT NOT NULL,
            playlist_uuid TEXT NOT NULL,
            item_key TEXT NOT NULL,
            payload TEXT NOT NULL,
            updated_at TIMESTAMP NOT NULL,
            PRIMARY KEY (kind, playlist_uuid, item_key)
        )",
        [],
    )?;

    Ok(())
}
