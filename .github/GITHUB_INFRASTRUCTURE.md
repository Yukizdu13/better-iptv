# GitHub Infrastructure Setup

This document explains the GitHub infrastructure setup for Better IPTV.

## 📋 Issue & PR Templates

### Bug Report Template

**File**: `.github/ISSUE_TEMPLATE/bug_report.yml`

Structured form for bug reports with the following fields:

- Bug description
- Steps to reproduce
- Expected vs actual behavior
- Operating system selection
- Version information
- MPV version
- Content type affected
- Log file upload
- Screenshots
- Additional context

**Benefits**:

- Consistent bug reports with all necessary information
- Easier to triage and reproduce issues
- Automatic labeling with `bug` and `needs-triage`

### Feature Request Template

**File**: `.github/ISSUE_TEMPLATE/feature_request.yml`

Structured form for feature requests with:

- Problem statement
- Proposed solution
- Alternative approaches
- Feature area selection
- Priority indication
- Examples from other apps
- Mockup/sketch upload
- Contribution willingness checkbox

**Benefits**:

- Clear feature proposals
- Better understanding of user needs
- Automatic labeling with `enhancement` and `needs-triage`

### Template Configuration

**File**: `.github/ISSUE_TEMPLATE/config.yml`

Configures:

- Blank issues (enabled)
- Links to GitHub Discussions
- Links to documentation

### Pull Request Template

**File**: `.github/PULL_REQUEST_TEMPLATE.md`

Comprehensive PR checklist including:

- Change description
- Type of change (bug fix, feature, etc.)
- Related issues
- Testing performed (OS-specific)
- Code quality checklist (lint, format, clippy, tests)
- Documentation updates

**Benefits**:

- Consistent PR format
- Ensures code quality checks
- Reminds contributors of testing requirements

## ⚙️ GitHub Actions Improvements

### Test Workflow

**File**: `.github/workflows/test.yml`

**Added**:

- **Concurrency control**: Cancels old builds when new commits are pushed
  ```yaml
  concurrency:
    group: ${{ github.workflow }}-${{ github.ref }}
    cancel-in-progress: true
  ```
- **NPM caching**: Speeds up `npm ci` by caching node_modules
  ```yaml
  cache: 'npm'
  ```
- **Job summary**: Generates a visual summary in GitHub Actions UI

**Benefits**:

- Faster builds (cached npm packages)
- No wasted CI time on outdated PRs
- Better overview of test results

### Release Workflow

**File**: `.github/workflows/release.yml`

**Added**:

- **Concurrency control**: Prevents multiple releases from running simultaneously
  ```yaml
  concurrency:
    group: ${{ github.workflow }}-${{ github.ref }}
    cancel-in-progress: false
  ```
- **NPM caching**: Speeds up release builds

**Benefits**:

- Faster release builds
- Prevents race conditions in releases

## 🤖 Dependabot Configuration

**File**: `.github/dependabot.yml`

Automated dependency updates with:

**NPM Dependencies** (weekly, Mondays):

- Groups dev dependencies together (types, eslint, vite)
- Groups Tauri packages together
- Groups React packages together
- Limits to 5 open PRs
- Labels: `dependencies`, `frontend`

**Cargo Dependencies** (weekly, Mondays):

- Groups Tauri crates together
- Groups serde crates together
- Limits to 5 open PRs
- Labels: `dependencies`, `backend`

**GitHub Actions** (weekly, Mondays):

- Updates action versions
- Limits to 3 open PRs
- Labels: `dependencies`, `ci`

**Benefits**:

- Automatic security updates
- Grouped updates reduce PR spam
- Consistent commit message format (`chore(deps):`)

## 🧹 Stale Issue Management

**File**: `.github/workflows/stale.yml`

Automatically manages inactive issues and PRs:

**Issues**:

- Marked stale after 60 days of inactivity
- Closed 14 days after being marked stale
- Exempt labels: `pinned`, `security`, `critical`

**Pull Requests**:

- Marked stale after 30 days of inactivity
- Closed 7 days after being marked stale
- Exempt labels: `pinned`, `security`, `critical`

**Benefits**:

- Keeps issue tracker clean
- Focuses attention on active issues
- Automatically removes outdated PRs

## 🏷️ Label System

**File**: `.github/labels.yml`

Comprehensive label schema (apply manually in GitHub Settings):

**Type Labels**:

- `bug`, `enhancement`, `documentation`, `question`

**Priority Labels**:

- `priority:critical`, `priority:high`, `priority:medium`, `priority:low`

**Status Labels**:

- `needs-triage`, `needs-investigation`, `in-progress`, `blocked`, `stale`

**Component Labels**:

- `frontend`, `backend`, `ci`, `dependencies`

**Area Labels**:

- `area:playback`, `area:playlist`, `area:epg`, `area:ui`, `area:database`

**Platform Labels**:

- `platform:linux`, `platform:windows`, `platform:macos`

**Special Labels**:

- `good first issue`, `help wanted`, `security`, `performance`, etc.

**How to Apply**:

1. Go to GitHub repo → Settings → Labels
2. Manually create each label with the specified color and description
3. Or use a tool like [github-label-sync](https://github.com/Financial-Times/github-label-sync):
   ```bash
   npm install -g github-label-sync
   github-label-sync --access-token YOUR_TOKEN mewset/better-ip-tv .github/labels.yml
   ```

## 📊 GitHub Settings Recommendations

### General Settings

- ✅ Enable Issues
- ✅ Enable Discussions (for community Q&A)
- ❌ Disable Wiki (use docs/ folder instead)
- ❌ Disable Projects (unless you use them)

### Branch Protection (main branch)

- ✅ Require pull request reviews before merging
- ✅ Require status checks to pass before merging:
  - `test` (Test Build workflow)
- ✅ Require branches to be up to date before merging
- ✅ Require linear history (optional, prevents merge commits)
- ✅ Do not allow bypassing the above settings

### Merge Settings

- ✅ Allow squash merging (recommended)
- ❌ Allow merge commits (creates messy history)
- ❌ Allow rebase merging (can be confusing)

### Discussion Categories (recommended)

1. **Announcements** - Project updates
2. **General** - General discussion
3. **Ideas** - Feature proposals
4. **Q&A** - User questions
5. **Show and Tell** - User showcases

## 🚀 Next Steps

1. **Review and commit these files**:

   ```bash
   git add .github/
   git commit -m "feat: add comprehensive GitHub infrastructure"
   git push
   ```

2. **Enable GitHub Discussions**:
   - Go to Settings → General → Features
   - Check "Discussions"

3. **Apply labels**:
   - Option A: Manually in Settings → Labels
   - Option B: Use github-label-sync (see above)

4. **Configure branch protection**:
   - Go to Settings → Branches → Add rule
   - Apply settings mentioned above

5. **Test the workflows**:
   - Create a test issue using the bug report template
   - Create a test PR and verify the template appears
   - Push to main and verify test workflow runs with caching

6. **Monitor Dependabot**:
   - PRs will start appearing on Mondays
   - Review and merge security updates promptly

## 💡 Tips

- **Issue Templates**: Users will see a selection screen when creating issues
- **Draft Releases**: Your release workflow creates draft releases - review before publishing
- **Stale Bot**: Adjust timings in `stale.yml` if 60/30 days is too long/short
- **Dependabot**: Review grouped PRs weekly, test thoroughly before merging

## 🔧 Customization

All files are easily customizable:

- **Adjust stale timings**: Edit `days-before-stale` in `stale.yml`
- **Change label colors**: Edit `.github/labels.yml`
- **Modify templates**: Edit `.github/ISSUE_TEMPLATE/*.yml`
- **Update workflows**: Edit `.github/workflows/*.yml`

---

**Last Updated**: 2025-12-16
