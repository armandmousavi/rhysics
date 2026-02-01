#!/bin/bash

# Script to build and export simulations to WASM

set -e

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Default target directory (can be overridden with EXPORT_TARGET_DIR env var)
DEFAULT_TARGET_DIR="${EXPORT_TARGET_DIR:-${HOME}/Documents/armandmousavi.github.io/rhysics}"

# CI mode skips interactive prompts
CI_MODE="${CI:-false}"

# Save current directory
ORIGINAL_DIR=$(pwd)

# ============================================================================
# HELPER FUNCTIONS
# ============================================================================

# Get all chapter numbers from source directory
get_source_chapters() {
    ls -d chapter_*/ 2>/dev/null | sed 's/chapter_\([0-9]*\)\//\1/' | sort -n
}

# Get all section numbers for a chapter from source directory
get_source_sections() {
    local chapter_num=$1
    ls -d "chapter_${chapter_num}/section_"*/ 2>/dev/null | sed 's/.*section_\([0-9]*\)\//\1/' | sort -n
}

# Get all simulation names for a section from source directory
get_source_simulations() {
    local chapter_num=$1
    local section_num=$2
    local section_path="chapter_${chapter_num}/section_${section_num}"
    
    for dir in "$section_path"/*/; do
        if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
            basename "$dir"
        fi
    done
}

# Get all chapters from target (exported) directory
get_target_chapters() {
    local target=$1
    if [ -d "$target" ]; then
        ls -d "$target"/chapter_*/ 2>/dev/null | sed 's/.*chapter_\([0-9]*\)\//\1/' | sort -n
    fi
}

# Get all sections for a chapter from target directory
get_target_sections() {
    local target=$1
    local chapter_num=$2
    local chapter_path="$target/chapter_${chapter_num}"
    
    if [ -d "$chapter_path" ]; then
        ls -d "$chapter_path"/section_*/ 2>/dev/null | sed 's/.*section_\([0-9]*\)\//\1/' | sort -n
    fi
}

# Get all simulations for a section from target directory
get_target_simulations() {
    local target=$1
    local chapter_num=$2
    local section_num=$3
    local section_path="$target/chapter_${chapter_num}/section_${section_num}"
    
    if [ -d "$section_path" ]; then
        for dir in "$section_path"/*/; do
            if [ -d "$dir" ] && [ -d "$dir/pkg" ]; then
                basename "$dir"
            fi
        done
    fi
}

# ============================================================================
# INDEX GENERATION FUNCTIONS
# ============================================================================

# Generate/regenerate section index based on all simulations present
generate_section_index() {
    local target=$1
    local chapter_num=$2
    local section_num=$3
    local section_path="$target/chapter_${chapter_num}/section_${section_num}"
    local section_index="$section_path/index.html"
    
    echo -e "${BLUE}  Updating section ${section_num} index...${NC}"
    
    # Build list of simulations
    local sim_links=""
    for sim in $(get_target_simulations "$target" "$chapter_num" "$section_num"); do
        sim_links="${sim_links}                <li><a href=\"${sim}/index.html\">${sim}</a></li>
"
    done
    
    cat > "$section_index" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chapter ${chapter_num}, Section ${section_num} - Rhysics</title>
    <link rel="stylesheet" href="../../../styles.css">
</head>
<body>
    <div class="container">
        <div id="content">
            <div class="back-link"><a href="../index.html">&larr; Back to Chapter ${chapter_num}</a></div>
            <h1>Chapter ${chapter_num}, Section ${section_num}</h1>
            <p class="subtitle">Simulations</p>
            <ul class="link-list">
${sim_links}            </ul>
        </div>
    </div>
</body>
</html>
EOF
}

# Generate/regenerate chapter index based on all sections present
generate_chapter_index() {
    local target=$1
    local chapter_num=$2
    local chapter_path="$target/chapter_${chapter_num}"
    local chapter_index="$chapter_path/index.html"
    
    echo -e "${BLUE}  Updating chapter ${chapter_num} index...${NC}"
    
    # Build list of sections
    local section_links=""
    for sec in $(get_target_sections "$target" "$chapter_num"); do
        section_links="${section_links}                <li><a href=\"section_${sec}/index.html\">Section ${sec}</a></li>
"
    done
    
    cat > "$chapter_index" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chapter ${chapter_num} - Rhysics</title>
    <link rel="stylesheet" href="../../styles.css">
</head>
<body>
    <div class="container">
        <div id="content">
            <div class="back-link"><a href="../index.html">&larr; Back to All Chapters</a></div>
            <h1>Chapter ${chapter_num}</h1>
            <p class="subtitle">Sections</p>
            <ul class="link-list">
${section_links}            </ul>
        </div>
    </div>
</body>
</html>
EOF
}

# Generate/regenerate root index based on all chapters present
generate_root_index() {
    local target=$1
    local root_index="$target/index.html"
    
    echo -e "${BLUE}  Updating root index...${NC}"
    
    # Build list of chapters
    local chapter_links=""
    for ch in $(get_target_chapters "$target"); do
        chapter_links="${chapter_links}                <li><a href=\"chapter_${ch}/index.html\">Chapter ${ch}</a></li>
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
${chapter_links}            </ul>
        </div>
    </div>
</body>
</html>
EOF
}

# Update all indexes (section, chapter, root) after an export
update_all_indexes() {
    local target=$1
    local chapter_num=$2
    local section_num=$3
    
    echo -e "${BLUE}Updating index files...${NC}"
    
    # Update section index
    generate_section_index "$target" "$chapter_num" "$section_num"
    
    # Update chapter index  
    generate_chapter_index "$target" "$chapter_num"
    
    # Update root index
    generate_root_index "$target"
}

# ============================================================================
# BUILD FUNCTIONS
# ============================================================================

# Build a single simulation
build_simulation() {
    local chapter_num=$1
    local section_num=$2
    local sim_name=$3
    local target=$4
    
    local sim_dir="chapter_${chapter_num}/section_${section_num}/${sim_name}"
    local output_dir="${target}/chapter_${chapter_num}/section_${section_num}/${sim_name}"
    
    echo -e "${BLUE}Building: ${sim_name}${NC}"
    echo "  Source: $sim_dir"
    echo "  Output: $output_dir"
    
    mkdir -p "$output_dir"
    
    cd "$ORIGINAL_DIR/$sim_dir"
    
    # Build with wasm-pack
    wasm-pack build --target web --out-dir "$output_dir/pkg" --release
    
    # Remove wasm-pack's .gitignore so pkg files can be committed to target repo
    rm -f "$output_dir/pkg/.gitignore"
    
    # Copy index.html if it exists
    if [ -f "index.html" ]; then
        cp index.html "$output_dir/index.html"
    fi
    
    cd "$ORIGINAL_DIR"
    
    echo -e "${GREEN}  ✓ ${sim_name} built successfully${NC}"
    echo ""
}

# Export a single simulation (with index updates)
export_single_simulation() {
    local chapter_num=$1
    local section_num=$2
    local sim_name=$3
    local target=$4
    
    build_simulation "$chapter_num" "$section_num" "$sim_name" "$target"
    update_all_indexes "$target" "$chapter_num" "$section_num"
}

# Export all simulations in a section
export_section() {
    local chapter_num=$1
    local section_num=$2
    local target=$3
    
    echo -e "${YELLOW}Exporting all simulations in Chapter ${chapter_num}, Section ${section_num}${NC}"
    echo ""
    
    local sim_count=0
    for sim in $(get_source_simulations "$chapter_num" "$section_num"); do
        build_simulation "$chapter_num" "$section_num" "$sim" "$target"
        ((sim_count++)) || true
    done
    
    if [ $sim_count -eq 0 ]; then
        echo -e "${RED}No simulations found in chapter_${chapter_num}/section_${section_num}${NC}"
        return 1
    fi
    
    update_all_indexes "$target" "$chapter_num" "$section_num"
    
    echo -e "${GREEN}Exported ${sim_count} simulation(s) from Section ${section_num}${NC}"
}

# Export all sections in a chapter
export_chapter() {
    local chapter_num=$1
    local target=$2
    
    echo -e "${YELLOW}Exporting all sections in Chapter ${chapter_num}${NC}"
    echo ""
    
    local section_count=0
    for sec in $(get_source_sections "$chapter_num"); do
        echo -e "${YELLOW}--- Section ${sec} ---${NC}"
        for sim in $(get_source_simulations "$chapter_num" "$sec"); do
            build_simulation "$chapter_num" "$sec" "$sim" "$target"
        done
        # Update section index after building all its simulations
        generate_section_index "$target" "$chapter_num" "$sec"
        ((section_count++)) || true
    done
    
    if [ $section_count -eq 0 ]; then
        echo -e "${RED}No sections found in chapter_${chapter_num}${NC}"
        return 1
    fi
    
    # Update chapter and root indexes
    generate_chapter_index "$target" "$chapter_num"
    generate_root_index "$target"
    
    echo ""
    echo -e "${GREEN}Exported ${section_count} section(s) from Chapter ${chapter_num}${NC}"
}

# Export everything
export_all() {
    local target=$1
    
    echo -e "${YELLOW}Exporting ALL chapters${NC}"
    echo ""
    
    local chapter_count=0
    for ch in $(get_source_chapters); do
        echo -e "${YELLOW}=== Chapter ${ch} ===${NC}"
        echo ""
        for sec in $(get_source_sections "$ch"); do
            echo -e "${YELLOW}--- Section ${sec} ---${NC}"
            for sim in $(get_source_simulations "$ch" "$sec"); do
                build_simulation "$ch" "$sec" "$sim" "$target"
            done
            generate_section_index "$target" "$ch" "$sec"
        done
        generate_chapter_index "$target" "$ch"
        ((chapter_count++)) || true
    done
    
    if [ $chapter_count -eq 0 ]; then
        echo -e "${RED}No chapters found${NC}"
        return 1
    fi
    
    generate_root_index "$target"
    
    echo ""
    echo -e "${GREEN}Exported ${chapter_count} chapter(s)${NC}"
}

# Regenerate all indexes without rebuilding
regenerate_indexes() {
    local target=$1
    
    echo -e "${YELLOW}Regenerating all index files...${NC}"
    echo ""
    
    for ch in $(get_target_chapters "$target"); do
        for sec in $(get_target_sections "$target" "$ch"); do
            generate_section_index "$target" "$ch" "$sec"
        done
        generate_chapter_index "$target" "$ch"
    done
    generate_root_index "$target"
    
    echo -e "${GREEN}All indexes regenerated!${NC}"
}

# ============================================================================
# MAIN MENU
# ============================================================================

show_menu() {
    echo "WASM Export Tool"
    echo "===================================="
    echo ""
    echo "What would you like to export?"
    echo ""
    echo "  1) Single simulation"
    echo "  2) All simulations in a section"
    echo "  3) All sections in a chapter"
    echo "  4) All chapters"
    echo "  5) Regenerate indexes"
    echo "  q) Quit"
    echo ""
}

# Get target directory from user
get_target_directory() {
    read -p "Enter target directory [$DEFAULT_TARGET_DIR]: " target_dir
    target_dir=${target_dir:-$DEFAULT_TARGET_DIR}
    
    # Convert to absolute path if relative
    if [[ "$target_dir" != /* ]]; then
        target_dir="${ORIGINAL_DIR}/${target_dir}"
    fi
    
    echo "$target_dir"
}

# List available items for selection
list_chapters() {
    echo ""
    echo "Available chapters:"
    for ch in $(get_source_chapters); do
        echo "  - Chapter $ch"
    done
    echo ""
}

list_sections() {
    local chapter_num=$1
    echo ""
    echo "Available sections in Chapter ${chapter_num}:"
    for sec in $(get_source_sections "$chapter_num"); do
        echo "  - Section $sec"
    done
    echo ""
}

list_simulations() {
    local chapter_num=$1
    local section_num=$2
    echo ""
    echo "Available simulations in Chapter ${chapter_num}, Section ${section_num}:"
    for sim in $(get_source_simulations "$chapter_num" "$section_num"); do
        echo "  - $sim"
    done
    echo ""
}

# ============================================================================
# COMMAND LINE INTERFACE
# ============================================================================

# Handle command line arguments for non-interactive use
if [ $# -ge 1 ]; then
    case "$1" in
        --all|-a)
            target=${2:-$DEFAULT_TARGET_DIR}
            export_all "$target"
            exit 0
            ;;
        --chapter|-c)
            if [ -z "$2" ]; then
                echo -e "${RED}Error: Chapter number required${NC}"
                echo "Usage: $0 --chapter <chapter_num> [target_dir]"
                exit 1
            fi
            chapter_num=$2
            target=${3:-$DEFAULT_TARGET_DIR}
            export_chapter "$chapter_num" "$target"
            exit 0
            ;;
        --section|-s)
            if [ -z "$2" ] || [ -z "$3" ]; then
                echo -e "${RED}Error: Chapter and section numbers required${NC}"
                echo "Usage: $0 --section <chapter_num> <section_num> [target_dir]"
                exit 1
            fi
            chapter_num=$2
            section_num=$3
            target=${4:-$DEFAULT_TARGET_DIR}
            export_section "$chapter_num" "$section_num" "$target"
            exit 0
            ;;
        --sim|--simulation)
            if [ -z "$2" ] || [ -z "$3" ] || [ -z "$4" ]; then
                echo -e "${RED}Error: Chapter, section, and simulation name required${NC}"
                echo "Usage: $0 --sim <chapter_num> <section_num> <sim_name> [target_dir]"
                exit 1
            fi
            chapter_num=$2
            section_num=$3
            sim_name=$4
            target=${5:-$DEFAULT_TARGET_DIR}
            
            sim_dir="chapter_${chapter_num}/section_${section_num}/${sim_name}"
            if [ ! -d "$sim_dir" ]; then
                echo -e "${RED}Error: Simulation directory $sim_dir does not exist${NC}"
                exit 1
            fi
            
            export_single_simulation "$chapter_num" "$section_num" "$sim_name" "$target"
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
            echo "  $0                                           Interactive mode"
            echo "  $0 --all [target_dir]                        Export everything"
            echo "  $0 --chapter <num> [target_dir]              Export entire chapter"
            echo "  $0 --section <ch> <sec> [target_dir]         Export entire section"
            echo "  $0 --sim <ch> <sec> <name> [target_dir]      Export single simulation"
            echo "  $0 --regen [target_dir]                      Regenerate indexes only"
            echo ""
            echo "Legacy format (still supported):"
            echo "  $0 <chapter> <section> <sim_name>            Export single simulation"
            echo ""
            exit 0
            ;;
        *)
            # Legacy format: chapter section sim_name
            if [ $# -eq 3 ]; then
                chapter_num=$1
                section_num=$2
                sim_name=$3
                
                sim_dir="chapter_${chapter_num}/section_${section_num}/${sim_name}"
                if [ ! -d "$sim_dir" ]; then
                    echo -e "${RED}Error: Simulation directory $sim_dir does not exist${NC}"
                    exit 1
                fi
                
                target=$(get_target_directory)
                export_single_simulation "$chapter_num" "$section_num" "$sim_name" "$target"
                exit 0
            else
                echo -e "${RED}Unknown option: $1${NC}"
                echo "Use --help for usage information"
                exit 1
            fi
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
            list_chapters
            read -p "Enter chapter number: " chapter_num
            
            if [ -z "$(get_source_sections "$chapter_num")" ]; then
                echo -e "${RED}No sections found in chapter_${chapter_num}${NC}"
                continue
            fi
            
            list_sections "$chapter_num"
            read -p "Enter section number: " section_num
            
            if [ -z "$(get_source_simulations "$chapter_num" "$section_num")" ]; then
                echo -e "${RED}No simulations found in chapter_${chapter_num}/section_${section_num}${NC}"
                continue
            fi
            
            list_simulations "$chapter_num" "$section_num"
            read -p "Enter simulation name: " sim_name
            
            sim_dir="chapter_${chapter_num}/section_${section_num}/${sim_name}"
            if [ ! -d "$sim_dir" ]; then
                echo -e "${RED}Error: Simulation directory $sim_dir does not exist${NC}"
                continue
            fi
            
            target=$(get_target_directory)
            echo ""
            export_single_simulation "$chapter_num" "$section_num" "$sim_name" "$target"
            ;;
        2)
            list_chapters
            read -p "Enter chapter number: " chapter_num
            
            if [ -z "$(get_source_sections "$chapter_num")" ]; then
                echo -e "${RED}No sections found in chapter_${chapter_num}${NC}"
                continue
            fi
            
            list_sections "$chapter_num"
            read -p "Enter section number: " section_num
            
            target=$(get_target_directory)
            echo ""
            export_section "$chapter_num" "$section_num" "$target"
            ;;
        3)
            list_chapters
            read -p "Enter chapter number: " chapter_num
            
            if [ -z "$(get_source_sections "$chapter_num")" ]; then
                echo -e "${RED}No sections found in chapter_${chapter_num}${NC}"
                continue
            fi
            
            target=$(get_target_directory)
            echo ""
            export_chapter "$chapter_num" "$target"
            ;;
        4)
            target=$(get_target_directory)
            echo ""
            export_all "$target"
            ;;
        5)
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
