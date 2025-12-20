/// Common utilities and components for all physics simulations
use bevy::prelude::*;

pub mod grid_camera;

/// Common camera setup for 2D simulations
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Common component for velocity
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);



/// Helper to create a window configuration for WASM
#[cfg(target_arch = "wasm32")]
pub fn default_window_plugin(title: &str) -> bevy::window::WindowPlugin {
    use bevy::window::{Window, WindowPlugin};
    
    WindowPlugin {
        primary_window: Some(Window {
            title: title.to_string(),
            canvas: Some("#bevy-canvas".to_string()),
            ..default()
        }),
        ..default()
    }
}

/// Helper to create a window configuration for native
#[cfg(not(target_arch = "wasm32"))]
pub fn default_window_plugin(title: &str) -> bevy::window::WindowPlugin {
    use bevy::window::{Window, WindowPlugin};
    
    WindowPlugin {
        primary_window: Some(Window {
            title: title.to_string(),
            resolution: (800, 600).into(),
            ..default()
        }),
        ..default()
    }
}

