# Changelog

All notable changes to Better IPTV will be documented in this file.

## [2.1.0] - 2025-11-20

### Added

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

### Fixed

- **Wayland/Hyprland Compatibility** - Resolved EGL_BAD_PARAMETER crash on Wayland systems
  - Pinned WebKit2GTK to stable version 2.44.1 in GitHub Actions build pipeline
  - Fixes compatibility issues on Arch Linux with Hyprland and other Wayland compositors

- **Dropdown Dark Mode Styling** - Fixed dropdown menu colors in dark theme
  - Added `dark:[color-scheme:dark]` CSS property to properly style native select elements
  - Dropdown now respects dark theme in all browsers

- **Non-Functional Setting Removed** - Removed "Remember Position" setting
  - Setting did nothing due to MPV being started with `--no-resume-playback` flag
  - Cleaned up associated MPV flags

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

[2.1.0]: https://github.com/mewset/better-ip-tv/compare/v2.0.1...v2.1.0
[2.0.1]: https://github.com/mewset/better-ip-tv/compare/v2.0.0...v2.0.1
[2.0.0]: https://github.com/mewset/better-ip-tv/releases/tag/v2.0.0
