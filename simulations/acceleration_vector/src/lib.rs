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
    pub initial_velocity: Vec2,
    pub initial_position: Vec3,
    pub launched: bool,
}

impl Default for ProjectileSettings {
    fn default() -> Self {
        Self {
            acceleration_function_x: "-x * (1 + 0.0001 * cos(x^2 + y^2))".parse().unwrap(),
            acceleration_function_y: "-y * (1 + 0.0001 * cos(x^2 + y^2))".parse().unwrap(),
            raw_acceleration_function_x: "-x * (1 + 0.0001 * cos(x^2 + y^2))".to_string(),
            raw_acceleration_function_y: "-y * (1 + 0.0001 * cos(x^2 + y^2))".to_string(),
            initial_velocity: Vec2::new(-60.0, 60.0),
            initial_position: Vec3::new(100.0, 0.0, 0.0),
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
fn predicted_trajectory(settings: &ProjectileSettings, seconds: i32) -> Vec<Vec2> {
    let Ok(accel_x_fn) = settings.acceleration_function_x.clone().bind2("x", "y") else {
        return Vec::new();
    };
    let Ok(accel_y_fn) = settings.acceleration_function_y.clone().bind2("x", "y") else {
        return Vec::new();
    };

    let mut trajectory = Vec::new();

    let mut position = settings.initial_position.truncate();
    let mut velocity = settings.initial_velocity;

    let dt = 1.0 / 60.0;
    let steps = seconds as usize * 60;

    for i in 0..steps {
        (position, velocity) = step_projectile(
            position,
            velocity,
            &accel_x_fn,
            &accel_y_fn,
            dt,
        );

        // sample once per second (or change this density)
        if i % 60 == 0 {
            trajectory.push(position);
        }
    }

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
    mut materials: ResMut<Assets<ColorMaterial>>,
    settings: Res<ProjectileSettings>,
) {
    // Spawn projectile at initial position
    commands.spawn((
        Projectile,
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 1.0, 0.0))),
        Transform::from_translation(settings.initial_position).with_scale(Vec3::splat(10.0)),
        Velocity(settings.initial_velocity),
    ));
}

fn step_projectile(
    position: Vec2,
    velocity: Vec2,
    accel_x_fn: &impl Fn(f64, f64) -> f64,
    accel_y_fn: &impl Fn(f64, f64) -> f64,
    dt: f32,
) -> (Vec2, Vec2) {
    let ax = accel_x_fn(position.x as f64, position.y as f64) as f32;
    let ay = accel_y_fn(position.x as f64, position.y as f64) as f32;

    // semi-implicit Euler
    let new_velocity = velocity + Vec2::new(ax, ay) * dt;
    let new_position = position + new_velocity * dt;

    (new_position, new_velocity)
}

fn apply_acceleration(
    mut query: Query<(&mut Velocity, &Launched, &Transform), With<Projectile>>,
    settings: Res<ProjectileSettings>,
    time: Res<Time>,
) {
    let Ok(accel_x_fn) = settings.acceleration_function_x.clone().bind2("x", "y") else {
        return;
    };
    let Ok(accel_y_fn) = settings.acceleration_function_y.clone().bind2("x", "y") else {
        return;
    };

    let dt = time.delta_secs();

    for (mut velocity, launched, transform) in &mut query {
        if launched.0 {
            let pos = transform.translation.truncate();
            let ax = accel_x_fn(pos.x as f64, pos.y as f64) as f32;
            let ay = accel_y_fn(pos.x as f64, pos.y as f64) as f32;

            velocity.0 += Vec2::new(ax, ay) * dt;
        }
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity, &Launched), With<Projectile>>,
    time: Res<Time>,
) {
    for (mut transform, velocity, launched) in &mut query {
        if launched.0 {
            transform.translation.x += velocity.0.x * time.delta_secs();
            transform.translation.y += velocity.0.y * time.delta_secs();
        }
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
            velocity.0 = settings.initial_velocity;
            transform.translation = settings.initial_position;
            launched.0 = false;
            
            let current_trajectory = predicted_trajectory(&settings, 100);
            for position in current_trajectory {
                commands.spawn((
                    Mesh2d(meshes.add(Circle::default())),
                    MeshMaterial2d(materials.add(Color::srgb(0.8, 0.7, 0.8))),
                    Transform::from_translation(Vec3::new(position.x, position.y, 0.0)).with_scale(Vec3::splat(5.0)),
                    TrajectoryMarker,
                ));
            }
        } else if !launched.0 {
            velocity.0 = settings.initial_velocity;
            transform.translation = settings.initial_position;
            launched.0 = true;
        }
    }
}
