#!/bin/bash

# Interactive script to create a new physics simulation

set -e

echo "Physics Simulation Creator"
echo "=============================="
echo ""

# Get simulation name (snake_case)
read -p "Enter simulation name in snake_case (e.g., orders_of_magnitude): " sim_name
if [ -z "$sim_name" ]; then
    echo "Error: Simulation name cannot be empty"
    exit 1
fi

# Get display title
read -p "Enter display title (e.g., Orders of Magnitude): " display_title
if [ -z "$display_title" ]; then
    display_title=$sim_name
fi

# Create directory structure under simulations/
full_dir="simulations/${sim_name}"

echo ""
echo "Creating simulation at: $full_dir"

if [ -d "$full_dir" ]; then
    echo "Error: Directory $full_dir already exists"
    exit 1
fi

mkdir -p "$full_dir/src"

# Create Cargo.toml
cat > "$full_dir/Cargo.toml" << EOF
[package]
name = "${sim_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
bevy = { workspace = true }
log = { workspace = true }
rhysics-common = { path = "../../common", features = ["egui"] }

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = { workspace = true }
web-sys = { workspace = true }

[lib]
crate-type = ["cdylib", "rlib"]

# Faster wasm-pack builds: -O1 instead of default (wasm-opt is the slow part, not Rust)
[package.metadata.wasm-pack.profile.release]
wasm-opt = ["-O1"]
EOF

# Create lib.rs
cat > "$full_dir/src/lib.rs" << EOF
use bevy::prelude::*;
use rhysics_common::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(default_window_plugin(
            "${display_title}"
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

fn setup(commands: Commands) {
    spawn_camera(commands);
    
    // TODO: Initialize your simulation here
    log::info!("${display_title} simulation started!");
}

fn update() {
    // TODO: Add your simulation logic here
}
EOF

# Create main.rs
cat > "$full_dir/src/main.rs" << EOF
// Native binary entry point
fn main() {
    ${sim_name}::run();
}
EOF

# Create index.html
cat > "$full_dir/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>${display_title}</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100vw;
            height: 100vh;
            overflow: hidden;
            background: #1a1a1a;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
        }
        header {
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            z-index: 100;
            display: flex;
            align-items: center;
            padding: 20px;
            background: #2a2a2a;
            color: #fff;
            box-shadow: 0 2px 10px rgba(0,0,0,0.3);
        }
        header h1 {
            flex: 1;
            margin: 0;
            font-size: 24px;
            font-weight: 300;
            text-align: center;
        }
        .back-link {
            color: #888;
            text-decoration: none;
            font-size: 14px;
        }
        .back-link:hover {
            color: #fff;
        }
        #canvas-container {
            margin-top: 68px;
            height: calc(100vh - 68px);
            display: flex;
            justify-content: center;
            align-items: center;
        }
        canvas {
            max-width: 100%;
            max-height: 100%;
        }
        #loading {
            color: #fff;
            font-size: 18px;
        }
    </style>
</head>
<body>
    <header>
        <a class="back-link" href="../index.html">&larr; Back</a>
        <h1>${display_title}</h1>
    </header>
    <div id="canvas-container">
        <div id="loading">Loading...</div>
        <canvas id="bevy-canvas" style="display:none;"></canvas>
    </div>
    <script type="module">
        import init from './pkg/${sim_name}.js';
        init().then(() => {
            document.getElementById('loading').style.display = 'none';
            document.getElementById('bevy-canvas').style.display = 'block';
        }).catch(err => {
            document.getElementById('loading').textContent = 'Error: ' + err;
        });
    </script>
</body>
</html>
EOF

echo ""
echo " Simulation created successfully!"
echo ""
echo " Next steps:"
echo "   1. Add the simulation to Cargo.toml workspace members:"
echo "      \"${full_dir}\","
echo ""
echo "   2. Edit ${full_dir}/src/lib.rs to implement your simulation"
echo ""
echo "   3. Test locally:"
echo "      cargo run -p ${sim_name}"
echo ""
echo "   4. Build for WASM:"
echo "      ./export.sh --sim ${sim_name}"
echo ""

# Ask if we should add to workspace
read -p "Would you like to add this to Cargo.toml workspace now? (y/n): " add_to_workspace

if [ "$add_to_workspace" = "y" ] || [ "$add_to_workspace" = "Y" ]; then
    if ! grep -q "\"${full_dir}\"" Cargo.toml; then
        sed -i.bak "/members = \[/,/\]/s|\]|    \"${full_dir}\",\n]|" Cargo.toml
        echo " Added to workspace members in Cargo.toml"
        rm Cargo.toml.bak 2>/dev/null || true
    else
        echo "Already in workspace members"
    fi
fi

echo ""
echo "Done"
