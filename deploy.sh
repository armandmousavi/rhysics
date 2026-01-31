#!/bin/bash

# Build and deploy all simulations locally, then push to both repos

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TARGET_REPO="${HOME}/Documents/armandmousavi.github.io"
TARGET_DIR="${TARGET_REPO}/rhysics"
GITHUB_REPO="git@github.com:armandmousavi/armandmousavi.github.io.git"

echo -e "${BLUE}=== Rhysics Deploy ===${NC}"
echo ""

# Clone target repo if it doesn't exist
if [ ! -d "$TARGET_REPO/.git" ]; then
    echo -e "${BLUE}Cloning target repo...${NC}"
    git clone "$GITHUB_REPO" "$TARGET_REPO"
else
    # Pull latest changes
    echo -e "${BLUE}Pulling latest from target repo...${NC}"
    cd "$TARGET_REPO"
    git pull --ff-only || true
    cd "$SCRIPT_DIR"
fi

# Build all simulations
echo ""
echo -e "${BLUE}Building all simulations...${NC}"
./export.sh --all "$TARGET_DIR"

# Commit and push rhysics repo
echo ""
echo -e "${BLUE}Pushing rhysics source changes...${NC}"
cd "$SCRIPT_DIR"
if ! git diff --quiet || ! git diff --staged --quiet; then
    git add -A
    git commit -m "Update simulations" --no-verify
    git push
    echo -e "${GREEN}✓ Pushed rhysics source${NC}"
else
    echo -e "${YELLOW}No source changes to commit${NC}"
fi

# Commit and push target repo
echo ""
echo -e "${BLUE}Pushing built simulations to GitHub Pages...${NC}"
cd "$TARGET_REPO"
git add rhysics/
if git diff --staged --quiet; then
    echo -e "${YELLOW}No simulation changes to commit${NC}"
else
    git commit -m "Update rhysics simulations"
    git push
    echo -e "${GREEN}✓ Pushed to GitHub Pages${NC}"
fi

echo ""
echo -e "${GREEN}=== Deploy complete! ===${NC}"
