# 📺 Better IPTV

**Modern, powerful IPTV player for Linux, Windows, and macOS**

Better IPTV is a cross-platform desktop application that delivers the ultimate IPTV experience. Built with Rust and Tauri for maximum performance, and MPV for reliable video playback.

---

## ✨ Features

### 🎬 Complete Content Support
- **Live TV** - Stream live channels with real-time EPG
- **Movies (VOD)** - Thousands of movies on demand
- **TV Series** - Binge-watch your favorite series with automatic playlist

### 📋 Flexible Playlist Management
- **M3U/M3U8 files** - Import from file or URL
- **Xtream Codes API** - Direct integration with your IPTV provider
- Support for large playlists (10,000+ channels) without performance issues

### 📺 EPG (Electronic Program Guide)
- Shows what's airing **right now** on live channels
- See what's coming up **next** in the schedule
- Automatic updating and synchronization
- XMLTV format with gzip compression support

### 🎯 Smart Navigation
- **Quick search** - Find channels instantly
- **Content tabs** - Filter by Live TV, Movies, or Series
- **Virtual scrolling** - Handle massive playlists smoothly
- **Favorite marking** - Star your favorite channels

### 🎨 Modern User Experience
- Dark and light theme
- Grid layout with channel logos
- Visual program information on each channel
- "Now Playing" bar with complete info
- Responsive design

### 📺 TV Series with Style
- Dedicated series view with seasons and episodes
- Visual episode images and descriptions
- **Automatic playlist** - Start an episode, the rest plays in order
- Complete metadata (plot, rating, genre)

### 🚀 Performance & Stability
- **MPV integration** - All video formats and codecs supported
- **Hardware acceleration** - Automatic GPU usage
- **Local database** - SQLite for fast access
- **Batch processing** - Fast import of large playlists
- **Cross-platform** - One codebase for all operating systems

---

## 📥 Installation

> **⚠️ Known Issue (Linux/Wayland):** Versions 2.0.0-2.0.1 may encounter `EGL_BAD_PARAMETER` error on newer systems running Wayland (especially Hyprland). This is a WebKit2GTK compatibility issue that has been fixed in the build pipeline. **Workaround:** Build locally (`npm run tauri build`) or wait for the next release (2.0.2+) which includes the fix.

### System Requirements

#### **MPV (Media Player)**
Better IPTV uses MPV for video playback. First, install MPV:

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install mpv

# Arch Linux
sudo pacman -S mpv

# Fedora
sudo dnf install mpv
```

**macOS:**
```bash
brew install mpv
```

**Windows:**
- Download from [mpv.io](https://mpv.io/installation/)
- Or use Chocolatey: `choco install mpv`

#### **Better IPTV App**

**Pre-built packages:**
1. Go to [Releases](https://github.com/mewset/better-ip-tv/releases)
2. Download the appropriate package for your system:
   - **Linux**: `.AppImage`, `.deb`, or `.rpm`
   - **Windows**: `.msi` or `.exe`
   - **macOS**: `.dmg`

**Linux - AppImage:**
```bash
chmod +x Better-IPTV.AppImage
./Better-IPTV.AppImage
```

**Build from source:**
```bash
# Clone repository
git clone https://github.com/mewset/better-ip-tv.git
cd better-ip-tv

# Install dependencies
npm install

# Build application
npm run tauri build

# Build output location: src-tauri/target/release/bundle/
```

---

## 🚀 Getting Started

### Step 1: First Launch
Launch Better IPTV. You'll be greeted with the setup screen where you can import your first playlist.

### Step 2: Import Playlist

#### **Option A: M3U/M3U8 File**
1. Click **"Import M3U Playlist"**
2. Enter a name for your playlist (e.g., "My IPTV")
3. Choose source:
   - **Local file**: Browse and select your .m3u or .m3u8 file
   - **URL**: Paste the link to your M3U file
4. Click **"Import"**
5. Wait while channels are imported

#### **Option B: Xtream Codes**
1. Click **"Import Xtream Playlist"**
2. Enter a name for your playlist
3. Fill in your Xtream credentials:
   - **Server URL**: `http://server.com:port`
   - **Username**: Your username
   - **Password**: Your password
4. Click **"Import"**
5. Wait while channels, movies AND series are imported

### Step 3: Configure EPG (Optional)
1. Open settings (gear icon)
2. Go to **EPG Settings**
3. Enter URL to your XMLTV EPG file
4. Click **"Fetch EPG"**
5. EPG data will update automatically going forward

### Step 4: Start Watching!
- Browse the channel list or use the search function
- Click **"Play"** to start a channel
- MPV player opens in a new window
- See program information directly on the channel card or in the "Now Playing" bar

---

## 💡 Usage

### Navigate Between Content Types
Use the tabs at the top to filter content:
- **All** - Show everything
- **Live TV** - Only live channels with EPG
- **Movies** - VOD movies
- **Series** - TV series

### Search for Channels
Use the search field to find channels quickly:
- Searches both channel names and group names
- Real-time filtering as you type
- Works together with content tabs

### Watch TV Series
1. Go to the **Series** tab
2. Click on a series to open the series view
3. Select season in the season selector
4. Click **"Play"** on an episode
5. All following episodes in the season are automatically queued

### Mark Favorites
- Click the star icon on a channel card
- Favorite channels display with a yellow star
- Use the favorite filter to show only favorites

### Change Theme
1. Open settings (gear icon)
2. Choose between **Light** or **Dark** mode
3. Theme is saved automatically

### Manage Multiple Playlists
- Import multiple M3U or Xtream playlists
- Switch between playlists in the setup screen
- Delete old playlists by clicking the trash icon

---

## 🎮 Keyboard Shortcuts

*(Works in the MPV player)*

- **Space** - Pause/Play
- **F** - Fullscreen
- **↑/↓** - Volume up/down
- **←/→** - Seek backward/forward (10 seconds)
- **M** - Mute/unmute
- **Q** - Close player
- **Esc** - Exit fullscreen

---

## ❓ Frequently Asked Questions (FAQ)

### Why won't MPV open?
**Answer:** MPV must be installed on your system. See [Installation](#-installation) above.

Verify that MPV is installed:
```bash
mpv --version
```

### Can I watch channels directly in the app?
**Answer:** No, Better IPTV uses MPV as an external player. This provides better codec support and performance, but video is shown in a separate MPV window.

### EPG data not showing?
**Answer:** Check that:
1. Your playlist contains EPG identifiers (tvg-id or tvg-name)
2. You have configured an EPG URL in settings
3. EPG data has been fetched (click "Fetch EPG" in settings)

### How large playlists are supported?
**Answer:** Better IPTV can handle 50.000+ channels without performance issues thanks to virtual scrolling and batch processing.

### Does it work with VPN?
**Answer:** Yes, Better IPTV works excellently with VPN. Make sure your VPN is active before starting streams.

### Are my Xtream credentials stored securely?
**Answer:** Yes, all credentials are stored locally in a SQLite database on your computer. Nothing is sent to external servers.

### Can I play local video files?
**Answer:** No, Better IPTV is designed solely for IPTV streams. Use MPV directly for local files.

---

## 🛠️ Troubleshooting

### Channels constantly buffering
- **Solution 1**: Check your internet connection
- **Solution 2**: Try another channel (could be server issue)
- **Solution 3**: Increase cache size in MPV (advanced)

### Series not importing from Xtream
- **Check**: That your Xtream provider supports series
- **Verify**: Username and password are correct
- **Try**: Import again if network was unstable

### App won't start
- **Linux**: Check that `.AppImage` has execute permissions
- **Windows**: Run as administrator
- **macOS**: Allow app in System Preferences > Security

---

## 🤝 Contributing to the Project

Better IPTV is open source and we welcome contributions!

### Report Bugs
Found a bug? [Create an issue](https://github.com/mewset/better-ip-tv/issues) with:
- Detailed description of the problem
- Steps to reproduce the bug
- Your operating system and version
- Screenshots if possible

### Suggest Features
Have an idea? [Open a feature request](https://github.com/mewset/better-ip-tv/issues) and describe:
- What feature you want
- Why it would be useful
- How you envision it working

### Contribute Code

#### Preparation
1. **Fork the repository**
   ```bash
   # Click "Fork" on GitHub
   git clone https://github.com/mewset/better-ip-tv.git
   cd better-ip-tv
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Create a feature branch**
   ```bash
   git checkout -b feature/my-new-feature
   ```

#### Development Environment
```bash
# Start development server
npm run tauri dev

# Run frontend tests
npm run test

# Run Rust tests
cd src-tauri
cargo test
```

#### Code Standards
- **TypeScript**: Follow ESLint configuration
- **Rust**: Use `rustfmt` and `clippy`
  ```bash
  cargo fmt
  cargo clippy
  ```
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)
  ```
  feat: add favorite filtering
  fix: correct EPG timezone bug
  docs: update README with FAQ
  ```

#### Submit Pull Request
1. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: description of change"
   ```

2. **Push to your fork**
   ```bash
   git push origin feature/my-new-feature
   ```

3. **Create Pull Request on GitHub**
   - Go to your fork on GitHub
   - Click "New Pull Request"
   - Describe your changes in detail
   - Link related issues if applicable

4. **Wait for code review**
   - Project owner will review your code
   - May request changes or improvements
   - When approved, your PR will be merged

### Community Guidelines
- Be respectful and inclusive
- Give constructive feedback
- Help other users in issues
- Document your changes

---

## 📄 License

GNU General Public License v2.0

Better IPTV is free software. You can use, modify, and distribute it under the terms of GPL v2.0.

**Why GPL v2.0?**
MPV is licensed under GPL v2+, which requires derivative works to also be GPL-licensed. We chose GPL v2.0 for compatibility and to promote open source.

See [LICENSE](./LICENSE) for full license text.

---

## 🙏 Thanks To

- **[MPV Project](https://mpv.io/)** - Fantastic media player with comprehensive codec support
- **[Tauri](https://tauri.app/)** - Cross-platform framework that makes this possible
- **[Open TV](https://github.com/Fredolx/open-tv)** - Inspiration and architectural reference
- **IPTV Community** - For standards and protocol support

---

## 📞 Contact & Support

- **GitHub Issues**: [Report bugs or suggest features](https://github.com/mewset/better-ip-tv/issues)
- **Discussions**: [GitHub Discussions](https://github.com/mewset/better-ip-tv/discussions)
- **Email**: support@better-iptv.com

---

**Made with ❤️ for IPTV enthusiasts**

*Better IPTV is not affiliated with any IPTV provider. You are responsible for using the service in accordance with local laws and your IPTV provider's terms.*
