#!/bin/bash

# Better IPTV Release Script
# Interactive script to create a new release

set -e  # Exit on error

echo "🚀 Better IPTV Release Creator"
echo "================================"
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

# Get current version
current_version=$(node -p "require('./package.json').version")
echo "📦 Current version: $current_version"
echo ""

# Calculate preview versions
patch_preview=$(node -p "const v=require('./package.json').version.split('.').map(Number);v[2]++;v.join('.')")
minor_preview=$(node -p "const v=require('./package.json').version.split('.').map(Number);v[1]++;v[2]=0;v.join('.')")
major_preview=$(node -p "const v=require('./package.json').version.split('.').map(Number);v[0]++;v[1]=0;v[2]=0;v.join('.')")

# Ask for version bump type
echo "Select version bump type:"
echo "  1) PATCH (bug fixes)        - $current_version → $patch_preview"
echo "  2) MINOR (new features)     - $current_version → $minor_preview"
echo "  3) MAJOR (breaking changes) - $current_version → $major_preview"
echo "  4) CUSTOM (specify version)"
echo ""
read -p "Enter choice (1-4): " choice

case $choice in
    1)
        bump_type="patch"
        ;;
    2)
        bump_type="minor"
        ;;
    3)
        bump_type="major"
        ;;
    4)
        read -p "Enter custom version (e.g., 1.2.3): " custom_version
        bump_type="$custom_version"
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

# Bump version
echo ""
echo "🔄 Bumping version..."
if [[ $choice -eq 4 ]]; then
    npm version "$custom_version" --no-git-tag-version
else
    npm version "$bump_type" --no-git-tag-version
fi

# Sync version to Cargo.toml and tauri.conf.json
echo "🔄 Syncing version to Cargo.toml and tauri.conf.json..."
npm run version:sync

new_version=$(node -p "require('./package.json').version")
echo "✅ Version updated to: $new_version"
echo ""

# Ask for release notes
echo "📝 Release notes (what's new in this version?):"
echo "   (Press Ctrl+D when done, or Ctrl+C to cancel)"
release_notes=$(cat)

if [ -z "$release_notes" ]; then
    release_notes="Release v$new_version"
fi

echo ""
echo "📋 Summary:"
echo "  Version: $current_version → $new_version"
echo "  Release notes:"
echo "  ---"
echo "$release_notes"
echo "  ---"
echo ""
read -p "Continue with release? (y/n) " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Release cancelled"
    # Revert version changes
    git checkout package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock 2>/dev/null || true
    exit 1
fi

# Commit version bump
echo "📝 Creating version commit..."
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock
git commit -m "chore: bump version to $new_version"

# Create git tag with release notes
echo "🏷️  Creating git tag v$new_version..."
git tag -a "v$new_version" -m "$release_notes"

echo ""
echo "✅ Release prepared successfully!"
echo ""
echo "📤 Next steps:"
echo "  1. Review the changes:"
echo "     git log -1"
echo "     git show v$new_version"
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
    echo "👍 You can push later with: git push --set-upstream origin main && git push --tags"
fi

echo ""
echo "🎉 Release v$new_version created!"
