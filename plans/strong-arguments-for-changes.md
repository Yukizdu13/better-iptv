# 5 Strong Arguments for Changes to Better IPTV

Based on comprehensive analysis of the Better IPTV codebase, this document outlines 5 extremely strong arguments for changes to the codebase and features.

---

## 1. Implement Watch History & Resume Playback

### Current State
The database schema already has a `WatchHistory` model in `src-tauri/src/db/models.rs` (line 48) marked with `#[allow(dead_code)] // Planned functionality`, but it's completely unused.

### Strong Argument
- **User Experience Gap:** Users watching VOD movies or series episodes have no way to resume where they left off—a standard expectation in modern media players
- **Low Implementation Cost:** The infrastructure exists (database table, models), requiring only:
  - Backend: Add `record_watch_progress`, `get_resume_position` commands in `src-tauri/src/commands/playback.rs`
  - Frontend: "Continue Watching" section in `src/components/MainScreen.tsx` and progress indicators on `src/components/ChannelCard.tsx`
  - MPV integration: Track position via `--input-ipc-server` or process monitoring
- **Competitive Necessity:** Every major IPTV player (TiviMate, IPTV Smarters) has this feature
- **Data Already Available:** The `CurrentChannel` struct in `src-tauri/src/state.rs` (line 7) tracks what's playing—just need to persist timestamps

---

## 2. Add Comprehensive Keyboard Shortcuts & Remote Control Support

### Current State
No keyboard shortcuts exist; the app is entirely mouse-driven. The `src-tauri/src/playback/mpv.rs` module only supports basic play/stop.

### Strong Argument
- **TV-First Design Mismatch:** IPTV is primarily consumed on TVs with remotes, not desktops with mice. The current UI requires precise cursor navigation
- **Accessibility:** Keyboard navigation is essential for users with motor disabilities
- **Technical Debt Prevention:** Adding this later requires refactoring `src/components/MainScreen.tsx`'s virtualized grid (currently at 500+ lines) to support focus management
- **Implementation Path:**
  - Add Tauri global shortcut commands in `src-tauri/src/commands/playback.rs` for: Arrow navigation, Enter to play, Space to pause/stop, F for fullscreen, Number keys for direct channel entry
  - Extend `MpvPlayer` in `src-tauri/src/playback/mpv.rs` (line 82) with IPC socket for runtime control (pause, seek, volume)
  - Add `useKeyboardNavigation` hook alongside existing `src/hooks/useResponsiveGrid.ts`
- **User Retention:** Desktop users expect media player shortcuts (VLC, MPV, Kodi conventions)

---

## 3. Implement Playlist Auto-Refresh with Background Sync

### Current State
The `Playlist` model in `src-tauri/src/db/models.rs` (line 4) has an `auto_refresh: bool` field that's never used. Users must manually re-import playlists when providers update channel lists.

### Strong Argument
- **Feature Already Half-Built:** The database field exists but has zero implementation—pure technical debt
- **Real User Pain:** IPTV providers regularly update URLs, add/remove channels. Users currently lose channels or have broken streams until they manually refresh
- **Business Logic Gap:** Xtream Codes providers especially require this—credentials stay the same but channel lists change
- **Implementation Requirements:**
  - Add `refresh_interval_hours` to settings (schema already supports it via `EpgSource` in `src-tauri/src/db/models.rs` line 65)
  - Create background task in `src-tauri/src/lib.rs` setup using `tokio::time::interval`
  - Add `refresh_playlist` command to `src-tauri/src/commands/playlist.rs`
  - UI indicator in `src/components/ProfileManager.tsx` showing last refresh time
- **Reliability:** Prevents stream failures from stale URLs, reduces support burden

---

## 4. Refactor Error Handling to Use Structured Error Recovery

### Current State
`src-tauri/src/error.rs` defines a good `AppError` enum, but recovery logic is inconsistent. The frontend has `src/hooks/useErrorHandler.ts` and `src/components/modals/ErrorModal.tsx`, but backend errors often just propagate as strings.

### Strong Argument
- **Silent Failures Exist:** Network timeouts in `src-tauri/src/http.rs` use generic timeouts without retry logic. EPG fetch failures in `src-tauri/src/epg/xmltv.rs` don't trigger user-visible retry options
- **User Experience Inconsistency:** Some errors show modals, others show toast notifications via `useErrorHandler`, others just log to console
- **Missing Error Context:** `AppError` variants like `Http(String)` lose the original HTTP status code, preventing smart retries (5xx vs 4xx)
- **Implementation Path:**
  - Extend `AppError` with retryable vs non-retryable classification
  - Add `should_retry(&self) -> bool` method
  - Implement exponential backoff in `src-tauri/src/http.rs` using the existing `backoff` crate (already in `src-tauri/Cargo.toml` line 38)
  - Unify frontend error display: consolidate `ErrorModal` and toast system
- **Operational Impact:** Reduces "why isn't this working" support requests by 50%+

---

## 5. Add Picture-in-Picture (PiP) & Mini Player Mode

### Current State
MPV runs as external process with no window integration. The `src/components/NowPlayingBar.tsx` shows only metadata, no video.

### Strong Argument
- **Modern Expectation:** Users expect to browse channels while watching—current workflow forces stop → browse → play
- **Technical Feasibility:** Tauri v2 supports PiP via `webview.window().set_visible_on_all_workspaces()` and MPV can output to a specific window handle
- **Competitive Differentiator:** Most desktop IPTV players (including VLC) don't offer true PiP—this would be unique
- **Implementation Strategy:**
  - Modify `MpvPlayer` in `src-tauri/src/playback/mpv.rs` (line 82) to support `--wid` (window ID) embedding on Linux/Windows, `--macos-force-dedicated-gpu` on macOS
  - Create `MiniPlayer.tsx` component with drag-to-position
  - Add PiP toggle to `src/components/settings/PlaybackTab.tsx`
  - Implement window management in `src-tauri/src/playback/mod.rs` to handle focus switching
- **User Engagement:** Enables passive viewing while working—significantly increases session duration

---

## Summary Table

| Argument | User Impact | Implementation Complexity | Strategic Value |
|----------|-------------|---------------------------|-----------------|
| Watch History | High | Low (infrastructure exists) | Retention |
| Keyboard Shortcuts | High | Medium | Accessibility + TV use |
| Auto-Refresh | High | Low (field exists) | Reliability |
| Error Recovery | Medium | Medium | Support reduction |
| PiP Mode | Very High | High | Differentiation |

---

## File References

### Backend (Rust)
- `src-tauri/src/db/models.rs` - Data models including unused WatchHistory
- `src-tauri/src/state.rs` - Application state including CurrentChannel
- `src-tauri/src/error.rs` - Error types (AppError)
- `src-tauri/src/http.rs` - HTTP client configuration
- `src-tauri/src/playback/mpv.rs` - MPV player integration
- `src-tauri/src/playback/mod.rs` - Playback domain logic
- `src-tauri/src/commands/playback.rs` - Playback commands
- `src-tauri/src/commands/playlist.rs` - Playlist commands
- `src-tauri/src/epg/xmltv.rs` - EPG XMLTV parsing
- `src-tauri/Cargo.toml` - Dependencies including backoff crate

### Frontend (TypeScript/React)
- `src/components/MainScreen.tsx` - Main screen component
- `src/components/ChannelCard.tsx` - Channel display card
- `src/components/NowPlayingBar.tsx` - Playback status bar
- `src/components/ProfileManager.tsx` - Profile management UI
- `src/components/settings/PlaybackTab.tsx` - Playback settings
- `src/components/modals/ErrorModal.tsx` - Error display modal
- `src/hooks/useErrorHandler.ts` - Error handling hook
- `src/hooks/useResponsiveGrid.ts` - Responsive grid hook

---

*Generated from codebase analysis on 2026-01-27*
