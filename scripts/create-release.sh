#!/bin/bash

# Better IPTV Release Creator
# Creates a release for the CURRENT version in package.json
# Use this when you've already updated the version and want to release it

set -e  # Exit on error

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root
cd "$PROJECT_ROOT"

echo "🚀 Better IPTV - Create Release for Current Version"
echo "===================================================="
echo ""

# Check if we're in a git repo
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Check for uncommitted changes
if ! git diff-index --quiet HEAD --; then
    echo "⚠️  Warning: You have uncommitted changes"
    echo ""
    git status --short
    echo ""
    read -p "Do you want to commit these changes first? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        read -p "Enter commit message: " commit_msg
        git add .
        git commit -m "$commit_msg"
        echo "✅ Changes committed"
    else
        echo "❌ Aborted. Please commit or stash your changes first."
        exit 1
    fi
fi

# Get current version from package.json
current_version=$(node -p "require('./package.json').version")
echo "📦 Current version in codebase: v$current_version"
echo ""

# Check if tag already exists
if git rev-parse "v$current_version" >/dev/null 2>&1; then
    echo "❌ Error: Tag v$current_version already exists!"
    echo "   Either:"
    echo "   1. Delete the existing tag: git tag -d v$current_version && git push origin :refs/tags/v$current_version"
    echo "   2. Bump the version first using: ./scripts/release.sh"
    exit 1
fi

# Verify version is synced across files
cargo_version=$(grep '^version = ' src-tauri/Cargo.toml | head -1 | cut -d'"' -f2)
tauri_version=$(node -p "require('./src-tauri/tauri.conf.json').version")

if [[ "$current_version" != "$cargo_version" ]] || [[ "$current_version" != "$tauri_version" ]]; then
    echo "⚠️  Warning: Version mismatch detected!"
    echo "   package.json:        $current_version"
    echo "   Cargo.toml:          $cargo_version"
    echo "   tauri.conf.json:     $tauri_version"
    echo ""
    read -p "Do you want to sync versions now? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🔄 Syncing versions..."
        npm run version:sync
        echo "✅ Versions synced"
        echo ""
    else
        echo "❌ Aborted. Please sync versions manually or use: npm run version:sync"
        exit 1
    fi
fi

# Show commits since last release
echo "📋 Changes since last release:"
echo "================================"
last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [ -z "$last_tag" ]; then
    echo "   (First release - showing last 20 commits)"
    git log --oneline --no-decorate | head -20
else
    echo "   Since $last_tag:"
    git log --oneline --no-decorate ${last_tag}..HEAD
fi
echo "================================"
echo ""

# Ask for release notes
echo "📝 Release notes for v$current_version:"
echo "   (Use the commits above as reference)"
echo "   (Press Ctrl+D when done, or Ctrl+C to cancel)"
echo ""
release_notes=$(cat)

if [ -z "$release_notes" ]; then
    release_notes="Release v$current_version"
fi

echo ""
echo "📋 Summary:"
echo "  Version to release: v$current_version"
echo "  Release notes:"
echo "  ---"
echo "$release_notes"
echo "  ---"
echo ""
read -p "Create release for v$current_version? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Release cancelled"
    exit 1
fi

# Create git tag with release notes
echo "🏷️  Creating git tag v$current_version..."
git tag -a "v$current_version" -m "$release_notes"

echo ""
echo "✅ Release tag created successfully!"
echo ""
echo "📤 Next steps:"
echo "  1. Review the tag:"
echo "     git show v$current_version"
echo ""
echo "  2. Push to GitHub (this will trigger automatic builds):"
echo "     git push && git push --tags"
echo ""
echo "  3. GitHub Actions will automatically:"
echo "     • Build packages for Linux, Windows, macOS"
echo "     • Create a draft release with all binaries"
echo "     • Add your release notes"
echo ""
echo "  4. Go to GitHub Releases and publish the draft"
echo ""
read -p "Push to GitHub now? (y/n) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📤 Pushing to GitHub..."

    # Check if upstream is set, if not set it
    if ! git rev-parse --abbrev-ref --symbolic-full-name @{u} > /dev/null 2>&1; then
        current_branch=$(git rev-parse --abbrev-ref HEAD)
        echo "Setting upstream for branch $current_branch..."
        git push --set-upstream origin "$current_branch"
    else
        git push
    fi

    # Push tags
    git push --tags

    echo ""
    echo "🎉 Done! Check GitHub Actions for build progress:"
    echo "   https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
else
    echo "👍 You can push later with:"
    echo "   git push && git push --tags"
fi

echo ""
echo "🎉 Release v$current_version created!"
echo ""
echo "💡 Next development cycle:"
echo "   Use ./scripts/release.sh to bump version for next development cycle"
