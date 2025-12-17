#!/bin/bash

# Download MPV Windows Build for Bundling
# This script downloads the latest MPV Windows build from sourceforge

set -e  # Exit on error

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🎬 MPV Windows Build Downloader"
echo "================================"
echo ""

# MPV version and download URL
# Format: mpv-x86_64-YYYYMMDD-git-HASH.7z
# Check latest at: https://sourceforge.net/projects/mpv-player-windows/files/64bit/
MPV_VERSION="${1:-20251214-git-f7be2ee}"  # Default to latest known, or use first argument
MPV_FILENAME="mpv-x86_64-${MPV_VERSION}.7z"
MPV_URL="https://sourceforge.net/projects/mpv-player-windows/files/64bit/${MPV_FILENAME}/download"
MPV_DIR="$PROJECT_ROOT/resources/mpv"
TEMP_FILE="/tmp/mpv-windows.7z"

echo "📦 MPV Build: $MPV_VERSION"
echo "📁 Install Directory: $MPV_DIR"
echo ""

# Check if 7z is installed
if ! command -v 7z &> /dev/null; then
    echo "❌ Error: 7z is not installed"
    echo "   Install with:"
    echo "   - macOS: brew install p7zip"
    echo "   - Linux: sudo apt-get install p7zip-full"
    exit 1
fi

# Create resources directory if it doesn't exist
mkdir -p "$PROJECT_ROOT/resources"

# Remove old MPV if exists
if [ -d "$MPV_DIR" ]; then
    echo "🗑️  Removing old MPV installation..."
    rm -rf "$MPV_DIR"
fi

# Download MPV
echo "📥 Downloading MPV ${MPV_VERSION}..."
curl -L "$MPV_URL" -o "$TEMP_FILE"

if [ ! -f "$TEMP_FILE" ]; then
    echo "❌ Error: Download failed"
    exit 1
fi

echo "✅ Download complete ($(du -h "$TEMP_FILE" | cut -f1))"
echo ""

# Extract MPV
echo "📦 Extracting MPV..."
7z x "$TEMP_FILE" -o"$PROJECT_ROOT/resources" -y > /dev/null

# Rename extracted folder to 'mpv'
EXTRACTED_DIR=$(find "$PROJECT_ROOT/resources" -maxdepth 1 -type d -name "mpv-x86_64-*" | head -1)
if [ -z "$EXTRACTED_DIR" ]; then
    echo "❌ Error: Could not find extracted MPV directory"
    exit 1
fi

mv "$EXTRACTED_DIR" "$MPV_DIR"

# Clean up
rm "$TEMP_FILE"

echo "✅ MPV extracted to $MPV_DIR"
echo ""

# Verify MPV files
echo "🔍 Verifying MPV installation..."
if [ ! -f "$MPV_DIR/mpv.exe" ]; then
    echo "❌ Error: mpv.exe not found"
    exit 1
fi

if [ ! -f "$MPV_DIR/mpv.com" ]; then
    echo "❌ Error: mpv.com not found"
    exit 1
fi

# Count DLL files
DLL_COUNT=$(find "$MPV_DIR" -name "*.dll" | wc -l)
echo "✅ Found mpv.exe"
echo "✅ Found mpv.com"
echo "✅ Found $DLL_COUNT DLL files"
echo ""

# Show total size
TOTAL_SIZE=$(du -sh "$MPV_DIR" | cut -f1)
echo "📊 Total MPV size: $TOTAL_SIZE"
echo ""

# Create .gitignore for resources directory
cat > "$PROJECT_ROOT/resources/.gitignore" <<'GITIGNORE_EOF'
# Ignore bundled MPV (downloaded by script)
mpv/

# But keep this .gitignore
!.gitignore
GITIGNORE_EOF

echo "✅ Created resources/.gitignore"
echo ""

echo "🎉 MPV Windows build ready for bundling!"
echo ""
echo "Next steps:"
echo "  1. Run 'npm run tauri build' to build Windows installer with bundled MPV"
echo "  2. The Windows installer will now include MPV automatically"
echo ""
echo "To download a different MPV build:"
echo "  ./scripts/download-mpv.sh 20251214-git-f7be2ee"
echo "  (Check https://sourceforge.net/projects/mpv-player-windows/files/64bit/ for latest)"
echo ""
