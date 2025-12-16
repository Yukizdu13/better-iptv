#!/bin/bash

# Better IPTV AUR Update Script
# Updates the AUR package to match a new GitHub release

set -e  # Exit on error

echo "🏗️ Better IPTV AUR Package Updater"
echo "===================================="
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: Must be run from project root directory"
    exit 1
fi

# Check if aur-repo exists
if [ ! -d "aur-repo" ]; then
    echo "❌ Error: aur-repo directory not found"
    echo "   Clone it first with:"
    echo "   git clone ssh://aur@aur.archlinux.org/better-iptv-bin.git aur-repo"
    exit 1
fi

# Get current version from package.json
current_version=$(node -p "require('./package.json').version")
echo "📦 Current project version: $current_version"

# Get current AUR version
cd aur-repo
aur_version=$(grep "^pkgver=" PKGBUILD | cut -d'=' -f2)
echo "📦 Current AUR version: $aur_version"
echo ""

# Check if versions match
if [ "$current_version" == "$aur_version" ]; then
    echo "⚠️  AUR package is already at version $current_version"
    read -p "Do you want to bump pkgrel instead? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        # Bump pkgrel
        current_rel=$(grep "^pkgrel=" PKGBUILD | cut -d'=' -f2)
        new_rel=$((current_rel + 1))
        sed -i "s/^pkgrel=.*/pkgrel=$new_rel/" PKGBUILD
        echo "✅ Bumped pkgrel from $current_rel to $new_rel"

        # Regenerate .SRCINFO
        makepkg --printsrcinfo > .SRCINFO

        # Commit and push
        git add PKGBUILD .SRCINFO
        git commit -m "Bump pkgrel to $new_rel"

        read -p "Push to AUR now? (y/n) " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            git push
            echo "🎉 AUR package updated!"
        fi
        exit 0
    else
        echo "❌ Aborted"
        exit 1
    fi
fi

# Ask for confirmation
echo "📋 This will update AUR package from $aur_version to $current_version"
read -p "Continue? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Aborted"
    exit 1
fi

# Check if GitHub release exists
echo "🔍 Checking if GitHub release exists..."
appimage_url="https://github.com/mewset/better-iptv/releases/download/v${current_version}/Better.IPTV_${current_version}_amd64.AppImage"

if ! curl -sI "$appimage_url" | grep -q "HTTP/2 302"; then
    echo "❌ Error: GitHub release v$current_version not found"
    echo "   Make sure you've published the release first!"
    echo "   URL: $appimage_url"
    exit 1
fi
echo "✅ Release found on GitHub"

# Download AppImage and calculate checksum
echo "📥 Downloading AppImage to calculate checksum..."
temp_file=$(mktemp)
curl -sL "$appimage_url" -o "$temp_file"

echo "🔐 Calculating SHA256 checksum..."
new_checksum=$(sha256sum "$temp_file" | cut -d' ' -f1)
rm "$temp_file"
echo "✅ Checksum: $new_checksum"

# Update PKGBUILD
echo "📝 Updating PKGBUILD..."
sed -i "s/^pkgver=.*/pkgver=$current_version/" PKGBUILD
sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
sed -i "s/^sha256sums=.*/sha256sums=('$new_checksum')/" PKGBUILD

echo "✅ PKGBUILD updated"

# Show diff
echo ""
echo "📋 Changes:"
git diff PKGBUILD

# Regenerate .SRCINFO
echo ""
echo "🔄 Regenerating .SRCINFO..."
makepkg --printsrcinfo > .SRCINFO
echo "✅ .SRCINFO updated"

# Test build (optional)
echo ""
read -p "Test build the package? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🏗️ Building package..."
    makepkg -f
    echo "✅ Build successful!"

    read -p "Install the package? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        makepkg -si --noconfirm
        echo "✅ Package installed!"
    fi
fi

# Commit
echo ""
echo "📝 Creating commit..."
git add PKGBUILD .SRCINFO
git commit -m "Update to v$current_version"
echo "✅ Commit created"

# Push to AUR
echo ""
read -p "Push to AUR now? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📤 Pushing to AUR..."
    git push
    echo ""
    echo "🎉 AUR package updated successfully!"
    echo ""
    echo "📦 Users can now install with:"
    echo "   yay -S better-iptv-bin"
    echo ""
    echo "🔗 Package page:"
    echo "   https://aur.archlinux.org/packages/better-iptv-bin"
else
    echo "👍 You can push later with:"
    echo "   cd aur-repo && git push"
fi

cd ..
echo ""
echo "✨ Done!"
