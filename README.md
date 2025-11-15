# Rhysics - Interactive Physics Simulations

A collection of educational physics simulations built with Rust and Bevy, following the OpenStax Physics textbook. Each simulation can be compiled to WebAssembly for deployment to static websites.

## Project Structure

```
rhysics-bevy/
├── common/              # Shared library for all simulations
│   └── src/
│       └── lib.rs       # Common components, systems, and utilities
├── chapter_X/           # Organized by textbook chapters
│   └── section_Y/       # Sections containing multiple simulations
│       └── simulation_name/  # Individual simulation crates
│           ├── Cargo.toml
│           ├── src/
│           │   ├── lib.rs   # Simulation logic + WASM entry point
│           │   └── main.rs  # Native binary entry point
│           └── index.html   # Web interface for the simulation
├── create-sim.sh        # Interactive script to create new simulations
├── export-sim.sh        # Script to build and export to WASM
└── Cargo.toml           # Workspace configuration

```

## Getting Started

### Prerequisites

- Rust (latest stable) - [Install](https://rustup.rs/)
- wasm-pack - `cargo install wasm-pack`
- A local web server for testing (e.g., Python's http.server)

### Running Simulations Locally

```bash
# Run a specific simulation natively
cargo run -p orders_of_magnitude

# Or from the simulation directory
cd chapter_1/section_1_orders_of_magnitude
cargo run
```

### Creating a New Simulation

Use the interactive creation script:

```bash
./create-new.sh
```

This will prompt you for:
- Chapter number (e.g., 1)
- Section number (e.g., 1)
- Simulation name in snake_case (e.g., units_and_standards)
- Display title (e.g., "Units and Standards")

Multiple simulations can exist in the same section.

The script will:
1. Create the directory structure
2. Generate boilerplate code
3. Create a Cargo.toml with proper dependencies
4. Optionally add it to the workspace

### Building for WASM

Use the export script to build and export a simulation:

```bash
# Interactive mode
./export.sh

# Or with arguments
./export.sh 1 1 orders_of_magnitude
```

This will:
1. Build the simulation with wasm-pack
2. Generate JavaScript bindings
3. Copy files to your target directory
4. Create index pages if needed

### Testing WASM Builds Locally

```bash
# Navigate to the output directory
cd ~/Documents/armandmousavi.github.io/rhysics

# Start a local server
python3 -m http.server 8000

# Open in browser
# http://localhost:8000/chapter_1/section_1/
```

## Common Library

The `common` crate provides shared functionality:

### Components
- `Position` - 2D position component
- `Velocity` - 2D velocity component
- `Acceleration` - 2D acceleration component

### Systems
- `spawn_camera` - Creates a 2D camera
- `project_positions` - Syncs Position to Transform
- `apply_velocity` - Updates position from velocity
- `apply_acceleration` - Updates velocity from acceleration

### Constants
- Physics constants (gravity, speed of light, etc.)

### Utilities
- `default_window_plugin(title)` - Creates proper window config for native/WASM

## Development Workflow

1. **Create a new simulation**
   ```bash
   ./create-new.sh
   ```

2. **Implement the simulation**
   - Edit `src/lib.rs` in your simulation directory
   - Use common components and systems from `rhysics-common`
   - Test with `cargo run -p your_simulation_name`

3. **Build for web**
   ```bash
   ./export.sh
   ```

4. **Deploy** to your static site
