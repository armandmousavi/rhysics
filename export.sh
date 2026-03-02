#!/bin/bash

# Script to build and export simulations to WASM

set -e

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Default target directory: rhysics/ inside a clone of armandmousavi.github.io (deploy.sh clones it if needed; override with EXPORT_TARGET_DIR)
SCRIPT_DIR_EXPORT="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
DEFAULT_TARGET_DIR="${EXPORT_TARGET_DIR:-${SCRIPT_DIR_EXPORT}/armandmousavi.github.io/rhysics}"

# CI mode skips interactive prompts
CI_MODE="${CI:-false}"

# Save current directory
ORIGINAL_DIR=$(pwd)

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

# Get all simulation names from source (simulations/*/)
get_source_simulations() {
    for dir in simulations/*/; do
        [ -d "$dir" ] || continue
        [ -f "${dir}Cargo.toml" ] || continue
        basename "$dir"
    done | sort
}

# Get all simulation names from target (directories with pkg subdir)
get_target_simulations() {
    local target=$1
    if [ ! -d "$target" ]; then
        return
    fi
    for dir in "$target"/*/; do
        [ -d "$dir" ] || continue
        [ -d "${dir}pkg" ] || continue
        basename "$dir"
    done | sort
}

# ============================================================================
# INDEX GENERATION
# ============================================================================

# Generate/regenerate root index listing all simulations
generate_root_index() {
    local target=$1
    local root_index="$target/index.html"
    
    echo -e "${BLUE}  Updating root index...${NC}"
    
    local sim_links=""
    for sim in $(get_target_simulations "$target"); do
        sim_links="${sim_links}                <li><a href=\"${sim}/index.html\">${sim}</a></li>
"
    done
    
    cat > "$root_index" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rhysics - Physics Simulations</title>
    <link rel="stylesheet" href="../styles.css">
</head>
<body>
    <div class="container">
        <div class="back-link"><a href="../index.html">&larr; Back to Home</a></div>
        <div id="content">
            <h1>Rhysics</h1>
            <p class="subtitle">My toy physics simulations</p>
            <ul class="link-list">
${sim_links}            </ul>
        </div>
    </div>
</body>
</html>
EOF
}

# ============================================================================
# BUILD FUNCTIONS
# ============================================================================

# Build a single simulation (skip wasm-pack if output is newer than source)
build_simulation() {
    if ! command -v wasm-pack >/dev/null 2>&1; then
        echo -e "${RED}Error: wasm-pack not found. Install it with: cargo install wasm-pack${NC}"
        exit 1
    fi

    local sim_name=$1
    local target=$2
    
    local sim_dir="simulations/${sim_name}"
    local output_dir="${target}/${sim_name}"
    
    # Skip wasm-pack if output exists and no source file is newer (avoids slow wasm-opt when nothing changed)
    if [ -d "$output_dir/pkg" ]; then
        wasm_file=$(find "$output_dir/pkg" -maxdepth 1 -name '*.wasm' -print -quit 2>/dev/null)
        if [ -n "$wasm_file" ] && [ -f "$wasm_file" ]; then
            if ! find "$ORIGINAL_DIR/$sim_dir" -type f -newer "$wasm_file" 2>/dev/null | grep -q .; then
                [ -f "$ORIGINAL_DIR/$sim_dir/index.html" ] && cp "$ORIGINAL_DIR/$sim_dir/index.html" "$output_dir/index.html"
                echo -e "${GREEN}  ✓ ${sim_name} (cached)${NC}"
                echo ""
                return
            fi
        fi
    fi

    echo -e "${BLUE}Building: ${sim_name}${NC}"
    echo "  Source: $sim_dir"
    echo "  Output: $output_dir"
    
    mkdir -p "$output_dir"
    
    cd "$ORIGINAL_DIR/$sim_dir"
    
    wasm-pack build --target web --out-dir "$output_dir/pkg" --release
    
    rm -f "$output_dir/pkg/.gitignore"
    
    if [ -f "index.html" ]; then
        cp index.html "$output_dir/index.html"
    fi
    
    cd "$ORIGINAL_DIR"
    
    echo -e "${GREEN}  ✓ ${sim_name} built successfully${NC}"
    echo ""
}

# Export a single simulation (with index update)
export_single_simulation() {
    local sim_name=$1
    local target=$2
    
    build_simulation "$sim_name" "$target"
    echo -e "${BLUE}Updating index...${NC}"
    generate_root_index "$target"
}

# Export all simulations
export_all() {
    local target=$1
    
    echo -e "${YELLOW}Exporting ALL simulations${NC}"
    echo ""
    
    local count=0
    for sim in $(get_source_simulations); do
        build_simulation "$sim" "$target"
        ((count++)) || true
    done
    
    if [ $count -eq 0 ]; then
        echo -e "${RED}No simulations found${NC}"
        return 1
    fi
    
    generate_root_index "$target"
    
    echo ""
    echo -e "${GREEN}Exported ${count} simulation(s)${NC}"
}

# Regenerate root index only
regenerate_indexes() {
    local target=$1
    
    echo -e "${YELLOW}Regenerating index...${NC}"
    generate_root_index "$target"
    echo -e "${GREEN}Done!${NC}"
}

# ============================================================================
# MAIN MENU
# ============================================================================

show_menu() {
    echo "WASM Export Tool"
    echo "===================================="
    echo ""
    echo "  1) Single simulation"
    echo "  2) All simulations"
    echo "  3) Regenerate index"
    echo "  q) Quit"
    echo ""
}

get_target_directory() {
    read -p "Enter target directory [$DEFAULT_TARGET_DIR]: " target_dir
    target_dir=${target_dir:-$DEFAULT_TARGET_DIR}
    if [[ "$target_dir" != /* ]]; then
        target_dir="${ORIGINAL_DIR}/${target_dir}"
    fi
    echo "$target_dir"
}

list_simulations() {
    echo ""
    echo "Available simulations:"
    for sim in $(get_source_simulations); do
        echo "  - $sim"
    done
    echo ""
}

# ============================================================================
# COMMAND LINE INTERFACE
# ============================================================================

if [ $# -ge 1 ]; then
    case "$1" in
        --all|-a)
            target=${2:-$DEFAULT_TARGET_DIR}
            export_all "$target"
            exit 0
            ;;
        --sim|--simulation)
            if [ -z "$2" ]; then
                echo -e "${RED}Error: Simulation name required${NC}"
                echo "Usage: $0 --sim <sim_name> [target_dir]"
                exit 1
            fi
            sim_name=$2
            target=${3:-$DEFAULT_TARGET_DIR}
            
            if [ ! -d "simulations/$sim_name" ] || [ ! -f "simulations/$sim_name/Cargo.toml" ]; then
                echo -e "${RED}Error: Simulation simulations/$sim_name does not exist${NC}"
                exit 1
            fi
            
            export_single_simulation "$sim_name" "$target"
            exit 0
            ;;
        --regen|--regenerate)
            target=${2:-$DEFAULT_TARGET_DIR}
            regenerate_indexes "$target"
            exit 0
            ;;
        --help|-h)
            echo "WASM Export Tool"
            echo ""
            echo "Usage:"
            echo "  $0                         Interactive mode"
            echo "  $0 --all [target_dir]      Export all simulations"
            echo "  $0 --sim <sim_name> [target_dir]  Export single simulation"
            echo "  $0 --regen [target_dir]    Regenerate index only"
            echo ""
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage"
            exit 1
            ;;
    esac
fi

# ============================================================================
# INTERACTIVE MODE
# ============================================================================

while true; do
    show_menu
    read -p "Select option: " choice
    echo ""
    
    case $choice in
        1)
            list_simulations
            read -p "Enter simulation name: " sim_name
            if [ -z "$sim_name" ]; then
                echo -e "${RED}Simulation name cannot be empty${NC}"
                continue
            fi
            if [ ! -d "simulations/$sim_name" ] || [ ! -f "simulations/$sim_name/Cargo.toml" ]; then
                echo -e "${RED}Error: Simulation $sim_name does not exist${NC}"
                continue
            fi
            target=$(get_target_directory)
            echo ""
            export_single_simulation "$sim_name" "$target"
            ;;
        2)
            target=$(get_target_directory)
            echo ""
            export_all "$target"
            ;;
        3)
            target=$(get_target_directory)
            echo ""
            regenerate_indexes "$target"
            ;;
        q|Q)
            echo "Goodbye!"
            exit 0
            ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            ;;
    esac
    
    echo ""
    echo -e "${GREEN}Done!${NC}"
    echo ""
    read -p "Press Enter to continue..."
    echo ""
done
