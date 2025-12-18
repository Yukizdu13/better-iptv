# Changelog

All notable changes to Better IPTV will be documented in this file.

## [2.3.0] - TBD

### Added

- **Parental Controls** - Comprehensive content restriction system with PIN protection
  - **PIN Protection**: Secure 4-6 digit PIN with Argon2 hashing
    - Backend: `set_parental_pin`, `verify_parental_pin`, `reset_parental_pin` commands
    - Argon2 password hashing (memory-hard, GPU-resistant)
    - Unique cryptographic salt per PIN
  - **Manual Channel Blocking**: Select specific channels to block
    - Virtualized channel selection modal for performance with 10,000+ channels
    - Search and bulk select/deselect functionality
    - Blocked channels stored as JSON array in settings
  - **Auto-Detection**: Automatic blocking of adult content
    - Regex patterns for +18, XXX, Adult, Erotic, Porn markers
    - Configurable toggle in Settings
  - **Category Blocking**: Block entire channel categories at once
  - **Three Visibility Modes**:
    - Hide: Blocked channels completely filtered from list (default)
    - Lock Icon: Shows channel with lock icon overlay (clickable to unlock)
    - Blur: Shows blurred channel with lock icon (clickable to unlock)
  - **PIN Verification Before Playback**: Blocked channels require PIN before streaming
    - Click on blocked channel (or lock overlay) triggers PIN modal
    - Correct PIN unlocks and starts playback
    - Incorrect PIN shows error, prevents playback
  - **Secure PIN Reset**: Must verify current PIN before resetting parental controls
  - **Session-Based Unlock**: Temporarily unlock with PIN (resets on app restart)
  - **Filter Integration**: Parental filter applied between category and search filters
  - Implementation:
    - Backend: 6 new Tauri commands in `src-tauri/src/commands.rs` (~140 lines)
    - Database: `delete_setting()` helper function
    - Frontend: Extended Zustand store with parental state
    - Utilities: `src/lib/parentalControls.ts` - Detection and filtering logic
    - Modals: `PinEntryModal.tsx`, `ChannelBlockingModal.tsx`
    - UI: Comprehensive Parental Controls section in Settings
    - Visual: Lock/blur overlay system in `ChannelCard.tsx`

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

- **Modal System - Replaced native browser dialogs**
  - Created reusable modal components: `ConfirmationModal.tsx` and `ErrorModal.tsx`
  - Replaced all `window.confirm()` and `alert()` calls across the application
  - Affected components:
    - `Settings.tsx`: PIN reset confirmation, save error handling
    - `ProfileManager.tsx`: Profile operations errors (5 instances)
    - `ChannelBlockingModal.tsx`: Save error handling
  - Features:
    - Consistent styling with app theme (light/dark mode support)
    - Customizable titles, messages, and button labels
    - Danger variant for destructive actions (red styling with AlertTriangle icon)
    - Proper z-index layering for nested modals
  - Implementation:
    - `ConfirmationModal`: Supports danger/primary variants, custom button text
    - `ErrorModal`: Red AlertCircle icon, single action button

- **EPG URL Settings - Removed hardcoded default**
  - EPG URL field now starts empty instead of pre-filled with `https://iptv-epg.org/files/epg-se.xml.gz`
  - Added helpful recommendation text with clickable link to iptv-epg.org
  - Prevents confusion for users whose Xtream providers include EPG data
  - Implementation: `Settings.tsx` lines 39-40 (state initialization) and lines 246-256 (help text)

- **MPV Path Resolution (Windows)** - New platform-specific logic
  - Windows: Checks `resources/mpv/mpv.exe` first, falls back to system PATH
  - macOS/Linux: Uses system MPV only (unchanged behavior)

### Performance

- **Channel Blocking Modal Optimization** - Virtualized rendering for large channel lists
  - Uses `@tanstack/react-virtual` for efficient DOM rendering
  - Renders only ~10-15 visible items instead of all channels
  - Smooth scrolling and instant search with 10,000+ channels
  - 68px estimated item height with 5-item overscan buffer

### Fixed

- **Parental Controls - Auto-detect now actually blocks channels**
  - Auto-detect toggle now scans all channels and adds adult content to blocked list when saving settings
  - Previously only filtered at runtime without persisting to blocked channels list
  - Now logs: `"Auto-detect found X additional adult channels"`
  - Implementation: `Settings.tsx` handleSave() scans channels using `isAdultContent()` when auto-detect enabled

- **Parental Controls - Visibility modes now work correctly**
  - "Lock Icon" and "Blur" modes now show channels with visual overlay
  - Previously all blocked channels were hidden regardless of visibility mode
  - Filter now only hides channels when `parentalVisibility === 'hide'`
  - Implementation: `MainScreen.tsx` line 110 - Added visibility mode check to parental filter

- **Parental Controls - Lock overlay now clickable**
  - Click anywhere on locked channel card to trigger PIN verification
  - Added visual hover feedback (opacity change) to indicate clickability
  - Tooltip: "Click to unlock with PIN"
  - Implementation: `ChannelCard.tsx` - Added onClick handler to parental overlay div

- **PIN Modal - State reset between uses**
  - PIN modal now resets all state (PIN input, error, isSubmitting) when reopened
  - Previously would show "Processing..." if reopened after successful verification
  - Added useEffect hook that resets state when `isOpen` changes to true
  - Also updated `resetForm()` to include `isSubmitting` state
  - Implementation: `PinEntryModal.tsx` lines 27-36 and line 86

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
