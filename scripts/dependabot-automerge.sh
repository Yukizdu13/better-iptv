#!/bin/bash
# Dependabot Auto-Merge Script
# Automatically merges safe Dependabot PRs and reports on what needs manual review

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🤖 Dependabot Auto-Merge Script${NC}"
echo -e "${BLUE}================================${NC}\n"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}❌ Error: GitHub CLI (gh) is not installed${NC}"
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if we're authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}❌ Error: Not authenticated with GitHub CLI${NC}"
    echo "Run: gh auth login"
    exit 1
fi

echo -e "${BLUE}📋 Fetching Dependabot PRs...${NC}\n"

# Get all open Dependabot PRs
PRS=$(gh pr list --author "app/dependabot" --json number,title,labels --state open)

if [ "$PRS" == "[]" ]; then
    echo -e "${GREEN}✅ No Dependabot PRs found!${NC}"
    exit 0
fi

# Counters
MERGED=0
SKIPPED=0
NEEDS_REVIEW=0

# Arrays to track PRs
MERGED_PRS=()
SKIPPED_PRS=()
REVIEW_PRS=()

echo -e "${BLUE}Processing PRs...${NC}\n"

# Process each PR
echo "$PRS" | jq -c '.[]' | while read -r pr; do
    NUMBER=$(echo "$pr" | jq -r '.number')
    TITLE=$(echo "$pr" | jq -r '.title')

    # Determine if it's safe to auto-merge
    SAFE=false
    REASON=""

    # Safe patterns (grouped updates, minor/patch, dev dependencies)
    if echo "$TITLE" | grep -qiE "(group|bump.*dependencies)"; then
        if echo "$TITLE" | grep -qiE "(dev-dependencies|github-actions)"; then
            SAFE=true
            REASON="Grouped dev dependencies or GitHub Actions"
        elif echo "$TITLE" | grep -qiE "patch|minor"; then
            SAFE=true
            REASON="Grouped minor/patch update"
        fi
    fi

    # Patch updates are usually safe (unless it's a major dependency)
    if echo "$TITLE" | grep -qiE "bump .* from [0-9]+\.[0-9]+\.[0-9]+ to [0-9]+\.[0-9]+\.[0-9]+"; then
        # Extract version numbers
        FROM_VERSION=$(echo "$TITLE" | grep -oE "from [0-9]+\.[0-9]+\.[0-9]+" | grep -oE "[0-9]+\.[0-9]+\.[0-9]+")
        TO_VERSION=$(echo "$TITLE" | grep -oE "to [0-9]+\.[0-9]+\.[0-9]+" | grep -oE "[0-9]+\.[0-9]+\.[0-9]+")

        FROM_MAJOR=$(echo "$FROM_VERSION" | cut -d. -f1)
        TO_MAJOR=$(echo "$TO_VERSION" | cut -d. -f1)

        if [ "$FROM_MAJOR" == "$TO_MAJOR" ]; then
            # Same major version = safe minor/patch
            if ! echo "$TITLE" | grep -qiE "(react|tauri|vite)"; then
                SAFE=true
                REASON="Patch/minor update (non-critical dependency)"
            fi
        fi
    fi

    # GitHub Actions updates are usually safe
    if echo "$TITLE" | grep -qiE "actions/|github-actions"; then
        SAFE=true
        REASON="GitHub Actions update"
    fi

    # Security updates should be reviewed but flagged
    if echo "$pr" | jq -r '.labels[].name' | grep -qi "security"; then
        SAFE=false
        REASON="⚠️  SECURITY UPDATE - Needs review but high priority"
    fi

    if [ "$SAFE" = true ]; then
        echo -e "${GREEN}✅ Auto-merging #$NUMBER: $TITLE${NC}"
        echo -e "   Reason: $REASON"

        # Approve and merge
        if gh pr review "$NUMBER" --approve --body "Auto-approved by dependabot-automerge.sh: $REASON" 2>/dev/null; then
            if gh pr merge "$NUMBER" --squash --auto 2>/dev/null; then
                MERGED=$((MERGED + 1))
                MERGED_PRS+=("#$NUMBER: $TITLE")
                echo -e "${GREEN}   ✓ Merged successfully${NC}\n"
            else
                echo -e "${YELLOW}   ⚠ Could not merge (might need CI to pass first)${NC}\n"
                SKIPPED=$((SKIPPED + 1))
                SKIPPED_PRS+=("#$NUMBER: $TITLE (CI pending)")
            fi
        else
            echo -e "${YELLOW}   ⚠ Could not approve PR${NC}\n"
            SKIPPED=$((SKIPPED + 1))
            SKIPPED_PRS+=("#$NUMBER: $TITLE (approval failed)")
        fi
    else
        echo -e "${YELLOW}⚠️  Needs manual review #$NUMBER: $TITLE${NC}"
        if [ -n "$REASON" ]; then
            echo -e "   Reason: $REASON"
        else
            echo -e "   Reason: Major update or critical dependency"
        fi
        NEEDS_REVIEW=$((NEEDS_REVIEW + 1))
        REVIEW_PRS+=("#$NUMBER: $TITLE")
        echo ""
    fi
done

# Summary
echo -e "\n${BLUE}================================${NC}"
echo -e "${BLUE}📊 Summary${NC}"
echo -e "${BLUE}================================${NC}\n"

if [ $MERGED -gt 0 ]; then
    echo -e "${GREEN}✅ Auto-merged: $MERGED PRs${NC}"
    for pr in "${MERGED_PRS[@]}"; do
        echo -e "   $pr"
    done
    echo ""
fi

if [ $SKIPPED -gt 0 ]; then
    echo -e "${YELLOW}⏸️  Skipped (CI pending): $SKIPPED PRs${NC}"
    for pr in "${SKIPPED_PRS[@]}"; do
        echo -e "   $pr"
    done
    echo ""
fi

if [ $NEEDS_REVIEW -gt 0 ]; then
    echo -e "${RED}👀 Needs manual review: $NEEDS_REVIEW PRs${NC}"
    for pr in "${REVIEW_PRS[@]}"; do
        echo -e "   $pr"
    done
    echo ""
    echo -e "${YELLOW}💡 Review these PRs manually:${NC}"
    echo -e "   gh pr list --author \"app/dependabot\""
    echo ""
fi

if [ $MERGED -eq 0 ] && [ $SKIPPED -eq 0 ] && [ $NEEDS_REVIEW -eq 0 ]; then
    echo -e "${GREEN}✨ All Dependabot PRs processed!${NC}"
fi

echo -e "${BLUE}================================${NC}"
