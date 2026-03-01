#!/bin/bash

# Script to test all simulations compile correctly

set -e

echo "🧪 Testing All Simulations"
echo "=========================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

failed=0
succeeded=0

echo -e "${BLUE}Checking workspace...${NC}"
if cargo check --workspace; then
    echo -e "${GREEN}Workspace check passed${NC}"
    echo ""
else
    echo -e "${RED}Workspace check failed${NC}"
    exit 1
fi

# Test each simulation individually (simulations/*/)
for sim_dir in simulations/*/; do
    if [ -d "$sim_dir" ] && [ -f "${sim_dir}Cargo.toml" ]; then
        package_name=$(grep '^name = ' "${sim_dir}Cargo.toml" | sed 's/name = "\(.*\)"/\1/')
        
        echo -e "${BLUE}Testing: $package_name${NC}"
        
        if cargo check -p "$package_name" 2>&1; then
            echo -e "${GREEN}$package_name: OK${NC}"
            ((succeeded++))
        else
            echo -e "${RED}$package_name: FAILED${NC}"
            ((failed++))
        fi
        echo ""
    fi
done

echo ""
echo "=========================="
echo "Test Summary"
echo "=========================="
echo -e "Succeeded: ${GREEN}$succeeded${NC}"
echo -e "Failed: ${RED}$failed${NC}"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All tests passed${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi
