#!/bin/bash
# Setup Git Hooks for Better IPTV
# Installs pre-commit hook that runs CI tests locally

set -e

echo "🔧 Setting up Git hooks..."

# Create .git/hooks directory if it doesn't exist
mkdir -p .git/hooks

# Create pre-commit hook
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
# Pre-commit hook: Run CI tests locally before allowing commit

echo "🧪 Running pre-commit checks..."
echo ""

# Run CI tests (without build for speed)
if ! ./scripts/ci-test-local.sh; then
    echo ""
    echo "❌ Pre-commit checks failed!"
    echo "   Fix the errors above before committing."
    echo ""
    echo "💡 To skip this check (not recommended):"
    echo "   git commit --no-verify"
    exit 1
fi

echo ""
echo "✅ Pre-commit checks passed!"
exit 0
EOF

# Make hook executable
chmod +x .git/hooks/pre-commit

echo "✅ Git hooks installed!"
echo ""
echo "📝 What was installed:"
echo "   - Pre-commit hook: Runs linting, formatting, and tests before commit"
echo ""
echo "💡 To bypass hooks (not recommended):"
echo "   git commit --no-verify"
echo ""
echo "🔧 To uninstall:"
echo "   rm .git/hooks/pre-commit"
