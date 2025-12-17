# Changelog

All notable changes to Better IPTV will be documented in this file.

## [2.3.0] - TBD

### Added

- **Bundled MPV for Windows** - MPV player now included in Windows installer
  - Windows installer size increased from ~6MB to ~100MB
  - Latest MPV Windows build bundled in `resources/mpv/` directory
  - MPV uses date-based builds (format: `mpv-x86_64-YYYYMMDD-git-HASH.7z`)
  - Automatic fallback to system MPV if bundled version fails
  - Eliminates need for manual MPV installation on Windows
  - Implementation:
    - `scripts/download-mpv.sh`: Downloads MPV Windows build (supports version argument)
    - `src-tauri/tauri.conf.json`: Bundles MPV in Windows resources
    - `src-tauri/src/mpv/player.rs`: `get_mpv_path()` checks bundled path first on Windows
    - `.github/workflows/release.yml`: Downloads MPV during Windows build
  - macOS and Linux still use system MPV (via Homebrew/package managers)

### Changed

- **MPV Path Resolution (Windows)** - New platform-specific logic
  - Windows: Checks `resources/mpv/mpv.exe` first, falls back to system PATH
  - macOS/Linux: Uses system MPV only (unchanged behavior)

## [2.2.0] - 2025-12-17

### Added

- **Category Quick-Access Bar** - New horizontal scrollable bar for filtering by provider categories
  - Backend: `get_channel_groups()` function in `src-tauri/src/db/operations.rs`
  - Tauri command: `get_channel_groups(playlist_id, content_type?)` returns unique categories
  - Frontend: New `CategoryBar` component with chip-style buttons
  - Zustand store: Added `categoryFilter`, `categories`, `setCategoryFilter`, `setCategories`
  - Auto-fetches categories when content type tab changes
  - Filter resets to "All" when switching content type tabs
  - Supports filtering within Live TV, Movies, and Series tabs independently

- **Provider Category Ordering** - Categories now display in the provider's original order
  - New `category_order` column in channels table stores provider's category position
  - Xtream API categories are fetched and indexed before streams
  - Database migration auto-adds column for existing installations
  - Categories sorted by `MIN(category_order)` instead of alphabetically

- **Category Tests** - 2 new Rust unit tests for `get_channel_groups()` function

### Changed

- Channel filtering logic now includes category filter layer between content type and search

### Fixed

- **Xtream Category Names** - Fixed missing category names from Xtream API
  - Xtream API returns `category_id` in streams but not `category_name`
  - Now fetches categories separately (`get_live_categories`, `get_vod_categories`, `get_series_categories`)
  - Builds category ID → name map before processing streams
  - Users need to re-import playlist for categories to appear

## [2.1.1] - 2025-12-02

### Added

- **Typed Error System (Rust)** - New `AppError` enum in `src-tauri/src/error.rs`
  - Structured error types: `Database`, `Http`, `InvalidInput`, `PlaylistNotFound`, `ChannelNotFound`, `Mpv`, `Parse`, `Epg`, `Io`, `Config`
  - JSON-serializable with `thiserror` + `serde` integration
  - All Tauri commands now return `Result<T, AppError>` instead of `Result<T, String>`
  - Input validation on all commands

- **Frontend Error Handling**
  - `src/types/errors.ts`: TypeScript types matching Rust `AppError`
  - `src/hooks/useErrorHandler.ts`: Toast-based error display with auto-dismiss
  - `SectionErrorBoundary` component for granular error isolation

- **Extracted React Hooks** - Improved code organization
  - `useChannelFilter`: Channel filtering and search logic
  - `useChannelPlayback`: MPV playback control
  - `useEpgData`: EPG fetching with automatic refresh

- **Extracted UI Components**
  - `ChannelCard`: Individual channel display
  - `ChannelHeader`: Page header with playlist info
  - `SearchBar`: Search input with keyboard handling
  - `ContentTypeTabs`: Live/VOD/Series tab navigation
  - `NowPlayingBar`: Current playback status display

- **Database Performance Indexes**
  ```sql
  CREATE INDEX idx_channels_playlist_id ON channels(playlist_id);
  CREATE INDEX idx_channels_epg_id ON channels(epg_id);
  CREATE INDEX idx_watch_history_channel_id ON watch_history(channel_id);
  ```

- **Test Coverage** - 76 automated tests total
  - 32 Rust unit tests (database operations, MPV player, error handling)
  - 44 Frontend tests (error types, hooks, component behavior)

### Changed

- **MPV Player Refactoring** (`src-tauri/src/mpv/player.rs`)
  - Extracted helper methods: `apply_default_args()`, `apply_playback_options()`, `log_command()`, `spawn_mpv()`
  - New `MpvPlaybackOptions` struct for cleaner API
  - ~40% reduction in code duplication

- **MainScreen Component** - Reduced from 800+ to ~400 lines through hook extraction

### Fixed

- Channel ID handling for virtual/temporary channels (now uses `id: -1`)
- Error messages now display Swedish translations for known error types

### Technical Debt

- Consolidated 15+ `.map_err(|e| e.to_string())` patterns into typed errors
- Merged 3 overlapping EPG `useEffect` hooks into single `useEpgData` hook
- Removed string-based error propagation throughout Rust backend

## [2.1.0] - 2025-11-21

### Added

- **Multi-Profile System** - Manage multiple IPTV playlists as profiles
  - Card-based UI in Settings for easy profile management
  - Create, rename, delete, and switch between profiles
  - Active profile indication with visual badge (blue highlight)
  - Automatic profile switching with channel reload
  - Setup component reusable as modal for creating profiles within Settings
  - Automatic migration for existing users (first playlist becomes active profile)
  - Warning modal when attempting to delete the last profile
  - Seamless profile switching preserves EPG and favorites data

- **Language Settings** - Choose default audio and subtitle languages from 19 supported languages
  - Settings stored as ISO language codes for MPV integration
  - Dropdown selectors in Settings > Language Settings
  - Languages: Swedish, English, Norwegian, Danish, Finnish, German, French, Spanish, Italian, Portuguese, Dutch, Polish, Russian, Arabic, Turkish, Japanese, Chinese, Korean, and "Original"
  - MPV receives `--alang=` and `--slang=` parameters for each stream

- **Comprehensive Logging System** - Persistent application logging for troubleshooting
  - Backend: `tauri-plugin-log` with automatic file rotation
  - Frontend: Unified logging across TypeScript/React components
  - Log files stored at platform-specific locations:
    - Linux: `~/.local/share/better-ip-tv/logs/better-ip-tv.log`
    - Windows: `%APPDATA%\com.m0s.better-ip-tv\logs\better-ip-tv.log`
    - macOS: `~/Library/Application Support/com.m0s.better-ip-tv/logs/better-ip-tv.log`
  - Debug level in development, Info level in production
  - 10MB file rotation with 1 archived log retained

- **EPG Fetch Optimization** - Conditional EPG fetching
  - Only fetches EPG data when URL actually changes
  - Prevents unnecessary network requests when saving other settings
  - Added debug logging to track EPG fetch decisions

- **Responsive Grid Layout** - Dynamic card layout that adapts to screen size
  - Cards scale automatically based on viewport dimensions
  - Columns adjust from 2 (mobile) to 7 (4K displays)
  - Card height optimized to show ~4 rows on any screen
  - Improved space utilization on large monitors
  - Smooth resize handling with debounced updates

### Fixed

- **Wayland/Hyprland Compatibility** - Resolved EGL_BAD_PARAMETER crash on Wayland systems
  - Pinned WebKit2GTK to stable version 2.44.1 in GitHub Actions build pipeline
  - Fixes compatibility issues on Arch Linux with Hyprland and other Wayland compositors

- **Dropdown Dark Mode Styling** - Fixed dropdown menu colors in dark theme
  - Added `dark:[color-scheme:dark]` CSS property to properly style native select elements
  - Dropdown now respects dark theme in all browsers

- **Default Tab Selection** - Live TV tab now selected by default
  - Changed default content type filter from "All" to "Live" for better UX
  - Users see live channels immediately when opening the app

- **Non-Functional Setting Removed** - Removed "Remember Position" setting
  - Setting did nothing due to MPV being started with `--no-resume-playback` flag
  - Cleaned up associated MPV flags

- **Credential Masking in Logs** - Sensitive data protection for bug reports
  - Masks Xtream username and password parameters in MPV logs
  - Prevents accidental credential leakage when sharing log files in issue reports
  - Uses regex to replace credentials with `***` while preserving log structure

- **Custom HTTP User-Agent** - Improved provider compatibility
  - All external HTTP requests now use proper user-agent: `Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Better-IPTV/2.1.0`
  - Shared HTTP client with connection pooling and reasonable timeouts (30s default)
  - Centralized HTTP client management for consistency
  - Prevents potential provider blocking of generic `reqwest/0.12.x` user-agent

### Changed

- Replaced all console logging with persistent file logging
  - Frontend: `console.*` → `logger.*` (23 replacements)
  - Backend: `println!`/`eprintln!` → log macros (27 replacements)

## [2.0.1] - 2025-11-15

### Fixed

- Version bumping in build pipeline
- AppImage permissions issue
- EGL display errors on Wayland systems

## [2.0.0] - 2025-11-10

### Added

- Initial stable release
- M3U/M3U8 playlist import
- Xtream Codes API support
- EPG (Electronic Program Guide) integration
- Live TV, Movies (VOD), and TV Series support
- Dark/light theme
- Favorites system
- Cross-platform support (Linux, Windows, macOS)
- GitHub Actions CI/CD
- AUR (Arch User Repository) package

[2.2.0]: https://github.com/mewset/better-ip-tv/compare/v2.1.1...v2.2.0
[2.1.1]: https://github.com/mewset/better-ip-tv/compare/v2.1.0...v2.1.1
[2.1.0]: https://github.com/mewset/better-ip-tv/compare/v2.0.1...v2.1.0
[2.0.1]: https://github.com/mewset/better-ip-tv/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/mewset/better-ip-tv/releases/tag/v2.0.0
