use bevy::prelude::*;
use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};
use rhysics_common::*;
use rhysics_common::grid_camera::GridCameraPlugin;
use meval::{Expr};

mod ui;
use ui::UiPlugin;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Resource)]
pub struct ProjectileSettings {
    pub raw_acceleration_function_x: String,
    pub raw_acceleration_function_y: String,
    pub acceleration_function_x: Expr,
    pub acceleration_function_y: Expr,
    pub launched: bool,
}

impl Default for ProjectileSettings {
    fn default() -> Self {
        Self {
            acceleration_function_x: "0.0".parse().unwrap(),
            acceleration_function_y: "0.0".parse().unwrap(),
            raw_acceleration_function_x: "0.0".to_string(),
            raw_acceleration_function_y: "0.0".to_string(),
            launched: false
        }
    }
}

#[derive(Component, Default)]
struct Collider;

#[derive(Component, Default)]
struct Launched(bool);

#[derive(Component)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>, Transform, Collider, Velocity, Launched)]
struct Projectile;

#[derive(Component)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>, Transform)]
struct TrajectoryMarker;

/// Predicts the trajectory for each second
fn predicted_trajectory(_settings: &ProjectileSettings, _seconds: i32) -> Vec<Vec2> {
    let trajectory = Vec::new();
    // TODO: Implement trajectory prediction
    // let v0 = settings.initial_velocity.0;
    // let a = Vec2::new(0.0, settings.gravitational_constant);
    
    // for t in 1..=seconds {
    //     let t = t as f32;
    //     // Kinematic equation: position = v0*t + 0.5*a*t^2
    //     let position = v0 * t + 0.5 * a * t * t;
    //     trajectory.push(position);
    // }
    trajectory
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(default_window_plugin(
            "Chapter 4.3 - Projectile Test"
        )))
        .init_resource::<ProjectileSettings>()
        .add_plugins((
            UiPlugin,
            GridCameraPlugin::default(),
        ))
        .add_systems(Startup, setup_projectile)
        .add_systems(
            Update,
            (despawn_trajectory_markers, update_launch)
                .chain()
                .run_if(resource_changed::<ProjectileSettings>)
        )
        .add_systems(
            FixedUpdate,
            (apply_acceleration, apply_velocity).chain()
        )
        .add_systems(Update, check_for_collisions)
        .run();
}

fn setup_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    // Spawn projectile at the origin
    commands.spawn((
        Projectile,
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_translation(Vec3::ZERO).with_scale(Vec3::splat(10.0)),
    ));
}

fn apply_acceleration(
    mut query: Query<(&mut Velocity, &Launched, &Transform), With<Projectile>>,
    settings: Res<ProjectileSettings>,
    time: Res<Time>,
) {
    // Bind the functions once per frame instead of per projectile
    // If binding fails (e.g., user is still typing), skip acceleration this frame
    let Ok(accel_x_fn) = settings.acceleration_function_x.clone().bind2("x", "y") else {
        return;
    };
    let Ok(accel_y_fn) = settings.acceleration_function_y.clone().bind2("x", "y") else {
        return;
    };
    
    for (mut velocity, launched, transform) in &mut query {
        // Only apply acceleration when launched
        if launched.0 {
            let x: f64 = transform.translation.x.into();
            let y: f64 = transform.translation.y.into();
            let acceleration_x = accel_x_fn(x, y);
            let acceleration_y = accel_y_fn(x, y);
            velocity.0.x += acceleration_x as f32 * time.delta_secs();
            velocity.0.y += acceleration_y as f32 * time.delta_secs();
        }
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

fn check_for_collisions(
    mut projectile_query: Query<(&mut Velocity, &Transform), With<Projectile>>,
    collider_query: Query<&Transform, (With<Collider>, Without<Projectile>)>,
) {
    for (mut projectile_velocity, projectile_transform) in &mut projectile_query {
        for collider_transform in &collider_query {
            let projectile_center = projectile_transform.translation.truncate();
            // Circle::default() has radius 0.5, so actual visual radius = 0.5 * scale
            let projectile_radius = 0.5 * projectile_transform.scale.x;
            let border_center = collider_transform.translation.truncate();
            let border_half_size = collider_transform.scale.truncate() / 2.;
            
            let collision = projectile_collision(
                BoundingCircle::new(projectile_center, projectile_radius),
                Aabb2d::new(border_center, border_half_size),
            );

            if let Some(collision) = collision {
                // Reflect the projectile's velocity when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // Reflect only if the velocity is in the opposite direction of the collision
                // This prevents the projectile from getting stuck inside the bar
                match collision {
                    Collision::Left => reflect_x = projectile_velocity.0.x > 0.0,
                    Collision::Right => reflect_x = projectile_velocity.0.x < 0.0,
                    Collision::Top => reflect_y = projectile_velocity.0.y < 0.0,
                    Collision::Bottom => reflect_y = projectile_velocity.0.y > 0.0,
                }

                // Reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    projectile_velocity.0.x = -projectile_velocity.0.x;
                }

                // Reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    projectile_velocity.0.y = -projectile_velocity.0.y;
                }
            }
        }
    }
}

// Returns `Some` if `projectile` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `projectile` hit.
fn projectile_collision(projectile: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !projectile.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(projectile.center());
    let offset = projectile.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn despawn_trajectory_markers(mut commands: Commands, query: Query<Entity, With<TrajectoryMarker>>) {
    for trajectory_entity in query.iter() {
        commands.entity(trajectory_entity).despawn();
    }
}

fn update_launch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<ProjectileSettings>,
    mut projectile_query: Query<(&mut Velocity, &mut Transform, &mut Launched), With<Projectile>>,
) {
    if let Ok((mut velocity, mut transform, mut launched)) = projectile_query.single_mut() {
        if !settings.launched {
            // Reset to origin
            velocity.0 = Vec2::ZERO;
            transform.translation = Vec3::ZERO;
            launched.0 = false;
            
            // Show trajectory preview when not launched
            let current_trajectory = predicted_trajectory(&settings, 10);
            for position in current_trajectory {
                commands.spawn((
                    Mesh2d(meshes.add(Circle::default())),
                    MeshMaterial2d(materials.add(Color::srgb(0.8, 0.7, 0.8))),
                    Transform::from_translation(Vec3::new(position.x, position.y, 0.0)).with_scale(Vec3::splat(5.0)),
                    TrajectoryMarker,
                ));
            }
        } else if !launched.0 {
            velocity.0 = Vec2::ZERO;
            launched.0 = true;
        }
    }
}
