# What's New in Better IPTV

A simple overview of new features and improvements.

---

## Version 2.3.0 (Coming Soon)

### New Features

**Parental Controls 🔒**
- Protect your family with PIN-protected content restrictions
- **Set a secure PIN** (4-6 digits) to control access to restricted content
- **Block specific channels** - Choose exactly which channels to restrict
  - Easy-to-use channel selection with search
  - Works smoothly even with thousands of channels
- **Auto-detect adult content** - Automatically blocks channels with +18, XXX, or Adult in the name
  - Now actually adds channels to your blocked list when you save settings!
- **Block entire categories** - Block all channels in categories like "Adult" at once
- **Three viewing modes**:
  - Hide blocked channels completely (default)
  - Show with a lock icon - Click the lock to unlock with PIN
  - Show blurred with a lock icon - Click to unlock with PIN
- **PIN required to watch blocked content** - Enter your PIN before any blocked channel can play
- **Secure PIN reset** - Must enter current PIN before you can reset parental controls
- **Temporary unlock** - Enter PIN to temporarily access blocked content (locks again when you restart the app)
- All settings easily managed from the Settings menu

Perfect for families who want to ensure kids only see appropriate content!

### Improvements

**Parental Controls Work Better Now! 🔧**
- **Auto-detect actually works** - When you enable auto-detect and save, the app now scans all your channels and adds adult content to the blocked list (it didn't do this before!)
- **Lock and Blur modes now show channels** - Previously these modes would hide channels just like "Hide" mode. Now they actually show the channels with a lock icon or blur effect, and you can click them to unlock!
- **Easier to unlock channels** - Just click anywhere on a locked channel card to enter your PIN and watch
- **PIN modal works smoothly** - Fixed a bug where the PIN entry would get stuck on "Processing..." if you unlocked multiple channels in a row

**No More MPV Installation on Windows! 🎉**
- Windows users no longer need to install MPV separately
- Everything you need is included in the installer
- Just download, install, and start watching - that's it!
- The app will automatically use the included video player
- If you already have MPV installed, the app will still work perfectly

**Note for Mac and Linux users**: You'll still install MPV the usual way (via Homebrew on Mac or your package manager on Linux). This change only affects Windows.

**Fixed Linux Wayland Display Issue 🐧**
- Fixed crash on startup for Linux users with Wayland display server
- The app now automatically works around a WebKit bug that caused "EGL_BAD_PARAMETER" errors
- No configuration needed - it just works!

---

## Version 2.2.0 (December 17, 2025)

### New Features

**Category Quick-Access Bar**
- New horizontal scrollable bar showing provider categories (Sweden, Norway, F1, etc.)
- Click any category chip to instantly filter channels
- Categories update automatically based on your selected content type (Live TV, Movies, Series)
- "All" button to show all channels in the current tab
- Categories now appear in your provider's original order (not alphabetically)

This makes it much faster to find channels from your favorite categories without having to search!

**Note**: If you had the app before this update, please re-import your playlist in Settings to see categories.

---

## Version 2.1.1 (December 2, 2025)

### Improvements

**Better Error Handling**
- Clearer and more informative error messages
- Errors in one part of the app no longer crash the whole application

**Faster Performance**
- Improved database performance for large channel lists
- Optimized code for a smoother experience

**Higher Quality**
- 76 new automated tests for improved stability
- Restructured code for easier maintenance and fewer bugs

---

## Version 2.1.0 (November 21, 2025)

### New Features

**Multiple Profiles**
- Add and manage multiple IPTV playlists as separate profiles
- Easily switch between different providers or setups
- Each profile keeps its own channels and favorites

**Language Preferences**
- Set your preferred audio language (19 languages available)
- Set your preferred subtitle language
- Includes Swedish, English, Norwegian, Danish, Finnish, German, French, Spanish, Italian, Portuguese, Dutch, Polish, Russian, Arabic, Turkish, Japanese, Chinese, and Korean

**Responsive Layout**
- Channel cards now adapt to your screen size
- Better experience on everything from phones to 8K monitors

### Improvements

**Better Compatibility**
- Fixed crashes on Wayland systems (Arch Linux with Hyprland, etc.)
- Works better with more IPTV providers

**Usability**
- Live TV is now the default view when opening the app
- Dark mode now works properly in all dropdown menus
- Removed confusing "Remember Position" setting that didn't work

**Privacy**
- Log files now hide your login credentials automatically
- Safe to share logs when reporting bugs

---

## Version 2.0.1 (November 15, 2025)

### Fixes
- Fixed AppImage not launching on some systems
- Fixed display errors on Wayland

---

## Version 2.0.0 (November 10, 2025)

### First Stable Release

- Import M3U/M3U8 playlists or use Xtream Codes
- Live TV, Movies, and TV Series support
- Electronic Program Guide (EPG)
- Mark channels as favorites
- Dark and light themes
- Works on Linux, Windows, and macOS
