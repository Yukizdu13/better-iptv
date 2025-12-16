# Scripts

Utility scripts for Better IPTV development, release management, and maintenance.

## 📦 Release Management

### release.sh

**Interactive release creation script** - Creates new versioned releases with automatic version synchronization.

#### Usage

```bash
./scripts/release.sh
```

#### What It Does

1. **Pre-flight checks:**
   - Verifies you're in a git repository
   - Checks for uncommitted changes (prompts to commit if needed)

2. **Version bumping:**
   - Shows current version
   - Offers 4 options:
     - **PATCH** - Bug fixes (1.0.0 → 1.0.1)
     - **MINOR** - New features (1.0.0 → 1.1.0)
     - **MAJOR** - Breaking changes (1.0.0 → 2.0.0)
     - **CUSTOM** - Manual version (e.g., 1.5.3)
   - Updates `package.json`
   - Automatically syncs to `Cargo.toml` and `tauri.conf.json`

3. **Release documentation:**
   - Shows commits since last release
   - Prompts for release notes
   - Creates annotated git tag with release notes

4. **GitHub integration:**
   - Commits version bump
   - Offers to push to GitHub (triggers CI/CD)
   - Provides next steps guidance

#### Example Session

```bash
$ ./scripts/release.sh

🚀 Better IPTV Release Creator
================================

📦 Current version: 2.2.0

Select version bump type:
  1) PATCH (bug fixes)        - 2.2.0 → 2.2.1
  2) MINOR (new features)     - 2.2.0 → 2.3.0
  3) MAJOR (breaking changes) - 2.2.0 → 3.0.0
  4) CUSTOM (specify version)

Enter choice (1-4): 2

🔄 Bumping version...
✅ Version updated to: 2.3.0

📋 Changes since last release:
================================
   Since v2.2.0:
   a1b2c3d feat: add new feature
   d4e5f6g fix: bug fix
================================

📝 Release notes (what's new in this version?):
Added awesome new feature
Fixed critical bug

✅ Release prepared successfully!

📤 Push to GitHub now? (y/n) y
🎉 Done! Check GitHub Actions for build progress
```

#### Requirements

- Node.js installed (for version detection)
- Git repository with at least one commit
- Clean working directory or willingness to commit changes

---

### sync-version.cjs

**Version synchronization utility** - Ensures version consistency across all project files.

#### Usage

```bash
npm run version:sync
# or directly:
node scripts/sync-version.cjs
```

#### What It Does

Reads version from `package.json` (source of truth) and updates:

- `src-tauri/Cargo.toml` - Rust package version
- `src-tauri/tauri.conf.json` - Tauri configuration version

This ensures all version numbers stay synchronized across the project.

#### When It Runs

- Automatically called by `release.sh`
- Can be run manually if versions get out of sync
- Should be run after any manual version changes

#### Example Output

```bash
$ npm run version:sync

🔄 Synchronizing version numbers...

📦 Source version (package.json): 2.3.0

✅ Updated Cargo.toml to version 2.3.0
✅ Updated tauri.conf.json to version 2.3.0

✨ Version synchronization complete!
```

---

### update-aur.sh

**AUR package updater** - Updates the Arch User Repository package after a GitHub release.

#### Usage

```bash
./scripts/update-aur.sh
```

#### Prerequisites

1. **AUR repository cloned:**

   ```bash
   git clone ssh://aur@aur.archlinux.org/better-iptv-bin.git aur-repo
   ```

2. **GitHub release published** with AppImage artifact
3. **SSH keys configured** for AUR access

#### What It Does

1. **Version detection:**
   - Reads version from `package.json`
   - Compares with current AUR package version

2. **Release verification:**
   - Checks if GitHub release exists
   - Verifies AppImage artifact is available

3. **Checksum calculation:**
   - Downloads AppImage
   - Calculates SHA256 checksum
   - Updates PKGBUILD with new checksum

4. **AUR update:**
   - Updates `PKGBUILD` (version, pkgrel, checksum)
   - Regenerates `.SRCINFO`
   - Optionally test-builds package
   - Commits and pushes to AUR

#### Example Session

```bash
$ ./scripts/update-aur.sh

🏗️ Better IPTV AUR Package Updater
====================================

📦 Current project version: 2.3.0
📦 Current AUR version: 2.2.0

📋 This will update AUR package from 2.2.0 to 2.3.0
Continue? (y/n) y

🔍 Checking if GitHub release exists...
✅ Release found on GitHub

📥 Downloading AppImage to calculate checksum...
🔐 Calculating SHA256 checksum...
✅ Checksum: abc123def456...

📝 Updating PKGBUILD...
✅ PKGBUILD updated

🔄 Regenerating .SRCINFO...
✅ .SRCINFO updated

Test build the package? (y/n) y
🏗️ Building package...
✅ Build successful!

📤 Pushing to AUR...
🎉 AUR package updated successfully!

📦 Users can now install with:
   yay -S better-iptv-bin
```

#### Notes

- Only run after publishing GitHub release
- Requires `makepkg` (Arch Linux build tool)
- Updates `pkgrel` to 1 for new versions
- Can bump `pkgrel` for PKGBUILD-only changes

---

## 🤖 Automation

### dependabot-automerge.sh

**Dependabot PR auto-merger** - Automatically merges safe Dependabot PRs and flags critical updates for review.

#### Usage

```bash
./scripts/dependabot-automerge.sh
```

#### Prerequisites

- **GitHub CLI** (`gh`) installed and authenticated

  ```bash
  # Install (if not already)
  # macOS: brew install gh
  # Linux: see https://cli.github.com/

  # Authenticate
  gh auth login
  ```

#### What It Does

**✅ Auto-merges (Safe):**

- Grouped dev dependencies (types, eslint, prettier, vite, vitest)
- Grouped minor/patch updates (react, tauri groups)
- GitHub Actions updates
- Patch/minor updates for non-critical dependencies

**⚠️ Requires Manual Review:**

- Major version updates (e.g., React 18 → 19)
- Updates to critical dependencies (react, tauri, vite core)
- Security updates (flagged for priority review)
- Individual ungrouped updates to core packages

**⏸️ Skips (Temporarily):**

- PRs where CI checks are still running
- PRs that can't be approved/merged yet

#### Example Output

```bash
$ ./scripts/dependabot-automerge.sh

🤖 Dependabot Auto-Merge Script
================================

📋 Fetching Dependabot PRs...

Processing PRs...

✅ Auto-merging #42: chore(deps): bump dev-dependencies group
   Reason: Grouped dev dependencies or GitHub Actions
   ✓ Merged successfully

⚠️  Needs manual review #43: chore(deps): bump react from 18.2.0 to 19.0.0
   Reason: Major update or critical dependency

================================
📊 Summary
================================

✅ Auto-merged: 1 PRs
   #42: chore(deps): bump dev-dependencies group

👀 Needs manual review: 1 PRs
   #43: chore(deps): bump react from 18.2.0 to 19.0.0

💡 Review these PRs manually:
   gh pr list --author "app/dependabot"

================================
```

#### Safety Features

- Never merges major version updates automatically
- Never merges updates to critical dependencies without review
- Flags security updates for priority manual review
- Only merges when CI checks would pass
- Provides detailed logging of all actions

#### Recommended Workflow

Run this script every Monday morning (or after Dependabot creates PRs):

```bash
# 1. Run auto-merge script
./scripts/dependabot-automerge.sh

# 2. Review remaining PRs manually
gh pr list --author "app/dependabot"

# 3. Test and merge critical updates
gh pr checkout <PR_NUMBER>
npm run lint && npm run test:run && npm run tauri build
gh pr merge <PR_NUMBER> --squash
```

#### Customization

Edit the script to adjust what's considered "safe":

- Lines 60-90: Safe pattern detection
- Add/remove dependency names from critical list
- Adjust version bump logic

---

## 🔧 Script Workflow Integration

### Complete Release Workflow

```bash
# 1. Create release
./scripts/release.sh
# Bumps version, syncs across files, creates tag

# 2. GitHub Actions automatically builds packages
# Wait for workflow to complete (~10-15 minutes)

# 3. Publish GitHub release from draft
# Go to: https://github.com/mewset/better-iptv/releases

# 4. Update AUR package
./scripts/update-aur.sh
# Updates Arch Linux package with new version
```

### Weekly Maintenance Workflow

```bash
# Monday morning: Handle Dependabot PRs
./scripts/dependabot-automerge.sh

# Review and test critical updates
gh pr list --author "app/dependabot"

# Manual merge if needed
gh pr checkout <PR_NUMBER>
npm run test:run
gh pr merge <PR_NUMBER> --squash
```

### Emergency Hotfix Workflow

```bash
# 1. Fix the bug
git checkout -b hotfix/critical-bug
# ... make changes ...
git commit -m "fix: critical bug"

# 2. Create patch release
./scripts/release.sh
# Select option 1 (PATCH)

# 3. Push and publish
# GitHub Actions will build automatically

# 4. Update AUR (if needed)
./scripts/update-aur.sh
```

---

## 📚 Script Dependencies

| Script | Dependencies | Optional |
|--------|-------------|----------|
| `release.sh` | git, node | - |
| `sync-version.cjs` | node | - |
| `update-aur.sh` | git, curl, makepkg, sha256sum | - |
| `dependabot-automerge.sh` | gh (GitHub CLI) | jq (for parsing) |

### Installing Dependencies

**macOS:**

```bash
brew install gh jq
```

**Linux (Arch):**

```bash
sudo pacman -S github-cli jq
```

**Linux (Debian/Ubuntu):**

```bash
# GitHub CLI
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null
sudo apt update
sudo apt install gh jq
```

---

## 🛡️ Best Practices

### Version Management

- **Always use `release.sh`** - Don't manually edit version numbers
- **Keep package.json as source of truth** - All other versions sync from it
- **Tag every release** - `release.sh` does this automatically
- **Use semantic versioning** - MAJOR.MINOR.PATCH

### Release Process

- **Test before releasing** - Run full test suite
- **Write clear release notes** - Users need to know what changed
- **Check CI before publishing** - Ensure builds succeed
- **Update AUR after publishing** - Keep Arch users up to date

### Dependency Management

- **Review major updates carefully** - Test thoroughly before merging
- **Keep dependencies up to date** - Run automerge weekly
- **Prioritize security updates** - Merge ASAP when flagged
- **Group related updates** - Dependabot config already does this

---

## 🐛 Troubleshooting

### release.sh Issues

**"Not in a git repository"**

- Run from project root: `cd /path/to/better-ip-tv`

**"Uncommitted changes"**

- Commit or stash changes first
- Or let the script commit them for you

### update-aur.sh Issues

**"aur-repo directory not found"**

```bash
git clone ssh://aur@aur.archlinux.org/better-iptv-bin.git aur-repo
```

**"GitHub release not found"**

- Publish the GitHub release first
- Wait a few minutes for CDN propagation

### dependabot-automerge.sh Issues

**"GitHub CLI (gh) is not installed"**

- Install from: https://cli.github.com/

**"Not authenticated with GitHub CLI"**

```bash
gh auth login
```

**"Could not approve PR"**

- Ensure you have write permissions to the repository
- Check if PR is already merged/closed

---

## 📝 Adding New Scripts

When adding new scripts to this directory:

1. **Make executable:** `chmod +x scripts/new-script.sh`
2. **Add shebang:** `#!/bin/bash` or `#!/usr/bin/env node`
3. **Add to this README** with:
   - Script name and purpose
   - Usage instructions
   - Example output
   - Requirements
4. **Follow naming convention:** `kebab-case.sh` or `kebab-case.cjs`
5. **Include error handling:** `set -e` for bash scripts
6. **Add helpful output:** Use emoji and colors for clarity

---

**Last Updated:** 2025-12-16
