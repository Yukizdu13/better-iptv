# Version Management

This project uses a **centralized version management system** with `package.json` as the single source of truth.

## How It Works

All version numbers are synchronized across:
- `package.json` (source of truth)
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

## Creating a New Release (The Easy Way)

### 🚀 One Command to Rule Them All

```bash
npm run release
```

This interactive script will:
1. ✅ Check for uncommitted changes (and help you commit them)
2. ✅ Ask you to choose version bump type (PATCH/MINOR/MAJOR/CUSTOM)
3. ✅ Show you a preview of the new version
4. ✅ Ask for release notes
5. ✅ Update all version files automatically
6. ✅ Create a git commit with the version bump
7. ✅ Create a git tag with your release notes
8. ✅ Optionally push to GitHub (triggers automatic builds)

### Example Session

```bash
$ npm run release

🚀 Better IPTV Release Creator
================================

📦 Current version: 0.1.0

Select version bump type:
  1) PATCH (bug fixes)        - 0.1.0 → 0.1.1
  2) MINOR (new features)     - 0.1.0 → 0.2.0
  3) MAJOR (breaking changes) - 0.1.0 → 1.0.0
  4) CUSTOM (specify version)

Enter choice (1-4): 2

🔄 Bumping version...
✅ Version updated to: 0.2.0

📝 Release notes (what's new in this version?):
   (Press Ctrl+D when done)
Added EPG support and series playlist feature

📋 Summary:
  Version: 0.1.0 → 0.2.0
  Release notes:
  ---
  Added EPG support and series playlist feature
  ---

Continue with release? (y/n) y

✅ Release prepared successfully!

Push to GitHub now? (y/n) y
📤 Pushing to GitHub...

🎉 Done! GitHub Actions will build packages automatically.
```

### What Happens Next?

After you push:
1. **GitHub Actions** automatically builds packages for:
   - Linux (AppImage, .deb, .rpm)
   - Windows (.msi, .exe)
   - macOS (.dmg, .app)

2. A **draft release** is created on GitHub with:
   - All binary packages attached
   - Your release notes
   - Version tag

3. You **review and publish** the release on GitHub

## Manual Version Management

If you need more control, you can use these commands:

### Automated Version Bump

```bash
# Patch version (0.1.0 → 0.1.1)
npm run version:patch

# Minor version (0.1.0 → 0.2.0)
npm run version:minor

# Major version (0.1.0 → 1.0.0)
npm run version:major
```

These commands will:
1. Update the version in `package.json`
2. Create a git commit with the version change
3. Create a git tag (e.g., `v0.1.1`)
4. Automatically sync all other version files

### Manual Version Sync

If you manually edit the version in `package.json`, run:

```bash
npm run version:sync
```

This will update `Cargo.toml` and `tauri.conf.json` to match.

## Current Version

**0.1.0**

## Version History

- **0.1.0** (2025-11-17) - Initial development version
  - M3U/M3U8 playlist import (file and URL)
  - Xtream Codes API integration
  - Live stream, VOD, and Series support
  - EPG (Electronic Program Guide) via XMLTV
  - Dark mode theme
  - Channel favorites
  - Series playlist playback
  - MPV integration with external process

## Important Notes

- **Never manually edit** version numbers in `Cargo.toml` or `tauri.conf.json`
- **Always use** `package.json` as the source of truth
- **Always run** `npm run version:sync` after manually editing `package.json` version
- The sync script is located at `scripts/sync-version.cjs`

## Semantic Versioning

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Incompatible API changes
- **MINOR** version: New functionality (backwards-compatible)
- **PATCH** version: Bug fixes (backwards-compatible)

Format: `MAJOR.MINOR.PATCH` (e.g., `1.2.3`)
