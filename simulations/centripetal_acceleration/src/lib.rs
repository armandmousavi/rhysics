use bevy::prelude::*;
use rhysics_common::*;
use rhysics_common::grid_camera::GridCameraPlugin;
mod ui;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::ui::UiPlugin;

#[derive(Resource)]
pub struct ProjectileSettings {
    pub radius: f32,
    pub velocity: f32,
    pub initial_angle: f32,
    pub launched: bool,
}

impl Default for ProjectileSettings {
    fn default() -> Self {
        Self {
            radius: 250.0,
            velocity: 150.0,
            initial_angle: 0.0,
            launched: false
        }
    }
}

#[derive(Component, Default)]
struct Collider;

#[derive(Component, Default)]
struct Launched(bool);

#[derive(Component)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>, Transform, Collider, Launched)]
struct Projectile;

#[derive(Component)]
#[require(Mesh2d, MeshMaterial2d<ColorMaterial>, Transform)]
struct TrajectoryMarker;

/// Predicts the trajectory for each second
fn predicted_trajectory(settings: &ProjectileSettings, seconds: i32) -> Vec<Vec2> {
    let mut trajectory = Vec::new();
    let theta0 = settings.initial_angle;
    let v = settings.velocity;
    let r = settings.radius;
    let w = v / r;
    
    for t in 1..=seconds {
        let t = t as f32;
        let theta = (theta0 + w * t) % (2.0 * std::f32::consts::PI);
        let position = Vec2::new(r * theta.cos(), r * theta.sin());
        trajectory.push(position);
    }
    trajectory
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(default_window_plugin(
            "Chapter 4.4 - Centripetal Acceleration"
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
            (apply_angular_velocity).chain()
        )
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
        Transform::from_translation(Vec3::new(settings.radius * settings.initial_angle.cos(), settings.radius * settings.initial_angle.sin(), 0.0)).with_scale(Vec3::splat(10.0)),
    ));
}

fn step_projectile(
    position: Vec2,
    radius: f32,
    velocity: f32,
    dt: f32,
) -> Vec2 {
    
    let w = velocity / radius;
    let theta = position.y.atan2(position.x);
    let new_theta = (theta + w * dt) % (2.0 * std::f32::consts::PI);
    // semi-implicit Euler
    let new_position = Vec2::new(radius * new_theta.cos(), radius * new_theta.sin());

    new_position
}

fn apply_angular_velocity(
    mut query: Query<(&mut Transform, &Launched), With<Projectile>>,
    settings: Res<ProjectileSettings>,
    time: Res<Time>,
) {
    for (mut transform, launched) in &mut query {
        if launched.0 {
            transform.translation = step_projectile(
                transform.translation.truncate(),
                settings.radius,
                settings.velocity,
                time.delta_secs()
            ).extend(0.0);
        }
    }
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
    mut projectile_query: Query<(&mut Transform, &mut Launched), With<Projectile>>,
) {
    if let Ok((mut transform, mut launched)) = projectile_query.single_mut() {
        if !settings.launched {
            transform.translation = Vec3::new(settings.radius * settings.initial_angle.cos(), settings.radius * settings.initial_angle.sin(), 0.0);
            launched.0 = false;
            
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
            transform.translation = Vec3::new(settings.radius * settings.initial_angle.cos(), settings.radius * settings.initial_angle.sin(), 0.0);
            launched.0 = true;
        }
    }
}
