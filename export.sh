#!/bin/bash

# Script to build and export a simulation to WASM

set -e

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default target directory
DEFAULT_TARGET_DIR="${HOME}/Documents/armandmousavi.github.io/rhysics"

echo "WASM Export"
echo "===================================="
echo ""

# Check if arguments provided or ask interactively
if [ $# -eq 3 ]; then
    chapter_num=$1
    section_num=$2
    sim_name=$3
else
    # Interactive mode
    read -p "Enter chapter number: " chapter_num
    read -p "Enter section number: " section_num
    read -p "Enter simulation name: " sim_name
fi

# Validate inputs
if [ -z "$chapter_num" ] || [ -z "$section_num" ] || [ -z "$sim_name" ]; then
    echo -e "${RED}Error: Missing required parameters${NC}"
    echo "Usage: $0 <chapter_num> <section_num> <sim_name>"
    echo "Example: $0 1 1 orders_of_magnitude"
    exit 1
fi

chapter_dir="chapter_${chapter_num}"
section_dir="${chapter_dir}/section_${section_num}"
sim_dir="${section_dir}/${sim_name}"

# Check if simulation exists
if [ ! -d "$sim_dir" ]; then
    echo -e "${RED}Error: Simulation directory $sim_dir does not exist${NC}"
    exit 1
fi

# Ask for target directory
read -p "Enter target directory [$DEFAULT_TARGET_DIR]: " target_dir
target_dir=${target_dir:-$DEFAULT_TARGET_DIR}

# Save current directory for absolute path conversion
ORIGINAL_DIR=$(pwd)

# Convert to absolute path if relative
if [[ "$target_dir" != /* ]]; then
    target_dir="${ORIGINAL_DIR}/${target_dir}"
fi

# Create output directory structure
output_dir="${target_dir}/chapter_${chapter_num}/section_${section_num}/${sim_name}"
mkdir -p "$output_dir"

# Convert output_dir to absolute path as well
output_dir=$(cd "$(dirname "$output_dir")" && pwd)/$(basename "$output_dir")

echo ""
echo -e "${BLUE}Building simulation: ${sim_name}${NC}"
echo "  Source: $sim_dir"
echo "  Output: $output_dir"
echo ""

# Build with wasm-pack
echo -e "${BLUE}📦 Building WASM package...${NC}"
cd "$sim_dir"

# Use wasm-pack to build with absolute path
wasm-pack build --target web --out-dir "$output_dir/pkg" --release

# Copy index.html if it exists
if [ -f "index.html" ]; then
    echo -e "${BLUE}📄 Copying index.html...${NC}"
    cp index.html "$output_dir/index.html"
fi

# Go back to original directory
cd "$ORIGINAL_DIR"

# Create or update section index
section_index="${target_dir}/chapter_${chapter_num}/section_${section_num}/index.html"
if [ ! -f "$section_index" ]; then
    echo -e "${BLUE}📄 Creating section index...${NC}"
    cat > "$section_index" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chapter ${chapter_num}, Section ${section_num} - Physics Simulations</title>
</head>
<body>
    <h1>Chapter ${chapter_num}, Section ${section_num} - Simulations</h1>
    <div class="simulations">
        <a href="${sim_name}/index.html" class="sim-card">
            <h2>${sim_name}</h2>
        </a>
    </div>
</body>
</html>
EOF
fi

# Create or update chapter index
chapter_index="${target_dir}/chapter_${chapter_num}/index.html"
if [ ! -f "$chapter_index" ]; then
    echo -e "${BLUE}📄 Creating chapter index...${NC}"
    cat > "$chapter_index" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Chapter ${chapter_num} - Physics Simulations</title>
</head>
<body>
    <h1>Chapter ${chapter_num} - Physics Simulations</h1>
    <div class="sections">
        <a href="section_${section_num}/index.html" class="section-card">
            <h2>Section ${section_num}</h2>
        </a>
    </div>
</body>
</html>
EOF
fi

# Create root index if it doesn't exist
root_index="${target_dir}/index.html"
if [ ! -f "$root_index" ]; then
    echo -e "${BLUE}📄 Creating root index...${NC}"
    cat > "$root_index" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rhysics - Physics Simulations</title>
</head>
<body>
    <h1>Rhysics</h1>
    <p class="subtitle">Interactive Physics Simulations with Rust and Bevy</p>
    <div class="chapters">
        <a href="chapter_${chapter_num}/index.html" class="chapter-card">
            <h2>Chapter ${chapter_num}</h2>
        </a>
    </div>
</body>
</html>
EOF
fi

echo ""
echo -e "${GREEN}✅ Export complete!${NC}"
echo ""
echo "📁 Files exported to: $output_dir"
echo ""
echo "🌐 To test locally:"
echo "   cd $output_dir"
echo "   python3 -m http.server 8000"
echo "   Open: http://localhost:8000"
echo ""
