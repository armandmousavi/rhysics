/// Common utilities and components for all physics simulations
use bevy::prelude::*;

/// Common camera setup for 2D simulations
pub fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Common component for positioning entities
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Position(pub Vec2);

/// Common component for velocity
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Velocity(pub Vec2);

/// Common component for acceleration
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Acceleration(pub Vec2);

/// Common physics constants
pub mod constants {
    /// Gravitational acceleration (m/s²)
    pub const GRAVITY: f32 = 9.81;
    
    /// Speed of light (m/s)
    pub const SPEED_OF_LIGHT: f32 = 299_792_458.0;
    
    /// Planck's constant (J⋅s)
    pub const PLANCK: f64 = 6.62607015e-34;
}

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

/// System to project Position components to Transform.translation
pub fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}

/// System to apply velocity to position
pub fn apply_velocity(mut entities: Query<(&mut Position, &Velocity)>) {
    for (mut position, velocity) in &mut entities {
        position.0 += velocity.0;
    }
}

/// System to apply acceleration to velocity
pub fn apply_acceleration(mut entities: Query<(&mut Velocity, &Acceleration)>) {
    for (mut velocity, acceleration) in &mut entities {
        velocity.0 += acceleration.0;
    }
}

