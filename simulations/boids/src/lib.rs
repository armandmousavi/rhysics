use bevy::math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rhysics_common::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

const BACKGROUND_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);
const BOID_DIAMETER: f32 = 5.;
const BORDER_THICKNESS: f32 = 10.0;
const MAX_SPEED: f32 = 300.0;           // Maximum velocity magnitude
const VIEW_RADIUS: f32 = 50.0;         // How far boids can "see" neighbors
const ALIGN_WEIGHT: f32 = 15.0;          // Steer towards average heading
const COHESION_WEIGHT: f32 = 15.0;       // Steer towards center of neighbors
const SEPARATION_WEIGHT: f32 = 17.0;     // Avoid crowding neighbors
const WINDOW_AVOIDANCE_DISTANCE: f32 = 10.0;   // Start avoiding when this close to border
const WINDOW_AVOIDANCE_WEIGHT: f32 = 30.0;     // How strongly to avoid borders
const MOUSE_ATTRACTION_WEIGHT: f32 = 30.0;  // Steer towards mouse cursor
const MOUSE_ATTRACTION_DISTANCE: f32 = 100.0; // Distance at which mouse attraction is applied
const BORDER_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(default_window_plugin("Chapter 0.0 - Boids")))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, (setup, setup_boids, setup_borders).chain())
        .add_systems(Update, (update_boids, check_for_collisions, apply_velocity).chain())
        .run();
}

#[derive(Component)]
struct Boid;

// Default must be implemented to define this as a required component for the Border component below
#[derive(Component, Default)]
struct Collider;

// This is a collection of the components that define a "Border" in our game
#[derive(Component)]
#[require(Sprite, Transform, Collider)]
struct Border;

/// Which side of the arena is this border located on?
enum BorderLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl BorderLocation {
    /// (x, y) dimensions of the border, used in `transform.scale()`
    fn size(&self, window_width: f32, window_height: f32) -> Vec2 {
        match self {
            BorderLocation::Left | BorderLocation::Right => {
                Vec2::new(BORDER_THICKNESS, window_height)
            }
            BorderLocation::Bottom | BorderLocation::Top => {
                Vec2::new(window_width, BORDER_THICKNESS)
            }
        }
    }

    fn position(&self, window_width: f32, window_height: f32) -> Vec2 {
        match self {
            // Position borders so their INNER EDGE is at the window edge (not their center)
            // This means the border center must be BORDER_THICKNESS/2 outside the window edge
            BorderLocation::Left => Vec2::new(- (window_width / 2.) - BORDER_THICKNESS / 2., 0.),
            BorderLocation::Right => Vec2::new(window_width / 2. + BORDER_THICKNESS / 2., 0.),
            BorderLocation::Bottom => Vec2::new(0., - (window_height / 2.) - BORDER_THICKNESS / 2.),
            BorderLocation::Top => Vec2::new(0., window_height / 2. + BORDER_THICKNESS / 2.),
        }
    }
}

impl Border {
    // This "builder method" allows us to reuse logic across our border entities,
    // making our code easier to read and less prone to bugs when we change the logic
    // Notice the use of Sprite and Transform alongside Border, overwriting the default values defined for the required components
    fn new(location: BorderLocation, window_width: f32, window_height: f32) -> (Border, Sprite, Transform) {
        (
            Border,
            Sprite::from_color(BORDER_COLOR, Vec2::ONE),
            Transform {
                // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                // This is used to determine the order of our sprites
                translation: location.position(window_width, window_height).extend(0.0),
                // The z-scale of 2D objects must always be 1.0,
                // or their ordering will be affected in surprising ways.
                // See https://github.com/bevyengine/bevy/issues/4149
                scale: location.size(window_width, window_height).extend(1.0),
                ..default()
            },
        )
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    log::info!("Boids simulation started!");
}

fn setup_borders(
    mut commands: Commands,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let window_width = window.width();
    let window_height = window.height();
    commands.spawn(Border::new(BorderLocation::Left, window_width, window_height));
    commands.spawn(Border::new(BorderLocation::Right, window_width, window_height));
    commands.spawn(Border::new(BorderLocation::Bottom, window_width, window_height));
    commands.spawn(Border::new(BorderLocation::Top, window_width, window_height));
}

fn setup_boids(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.single() else {
        return;
    };
    let window_width = window.width();
    let window_height = window.height();
    // Spawn boids in random positions in window
    let spawn_width = window_width - BOID_DIAMETER * 2.;
    let spawn_height = window_height - BOID_DIAMETER * 2.;
    for _ in 0..1000 {
        commands.spawn((
            Mesh2d(meshes.add(Circle::default())),
            MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
            Transform::from_translation(Vec3::new(
                rand::random::<f32>() * spawn_width as f32 - spawn_width as f32 / 2.0,
                rand::random::<f32>() * spawn_height as f32 - spawn_height as f32 / 2.0,
                0.0,
            )).with_scale(Vec3::splat(BOID_DIAMETER / 2.)),
            Velocity(Vec2::new(
                rand::random::<f32>() * 400.0 - 200.0,
                rand::random::<f32>() * 400.0 - 200.0,
            )),
            Boid,
        ));
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.0.x * time.delta_secs();
        transform.translation.y += velocity.0.y * time.delta_secs();
    }
}

fn check_for_collisions(
    mut boid_query: Query<(&mut Velocity, &Transform), With<Boid>>,
    collider_query: Query<&Transform, With<Collider>>,
) {
    for (mut boid_velocity, boid_transform) in &mut boid_query {
        for collider_transform in &collider_query {
            let boid_center = boid_transform.translation.truncate();
            // Circle::default() has radius 0.5, so actual visual radius = 0.5 * scale
            let boid_radius = 0.5 * boid_transform.scale.x;
            let border_center = collider_transform.translation.truncate();
            let border_half_size = collider_transform.scale.truncate() / 2.;
            
            let collision = boid_collision(
                BoundingCircle::new(boid_center, boid_radius),
                Aabb2d::new(border_center, border_half_size),
            );

            if let Some(collision) = collision {
                // Reflect the boids's velocity when it collides
                let mut reflect_x = false;
                let mut reflect_y = false;

                // Reflect only if the velocity is in the opposite direction of the collision
                // This prevents the boid from getting stuck inside the bar
                match collision {
                    Collision::Left => reflect_x = boid_velocity.0.x > 0.0,
                    Collision::Right => reflect_x = boid_velocity.0.x < 0.0,
                    Collision::Top => reflect_y = boid_velocity.0.y < 0.0,
                    Collision::Bottom => reflect_y = boid_velocity.0.y > 0.0,
                }

                // Reflect velocity on the x-axis if we hit something on the x-axis
                if reflect_x {
                    boid_velocity.0.x = -boid_velocity.0.x;
                }

                // Reflect velocity on the y-axis if we hit something on the y-axis
                if reflect_y {
                    boid_velocity.0.y = -boid_velocity.0.y;
                }
            }
        }
    }
}

fn update_boids(
    mut query: Query<(&mut Boid, &mut Transform, &mut Velocity)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    // Get window dimensions and mouse position
    let Ok(window) = window_query.single() else {
        return;
    };
    let window_width = window.width();
    let window_height = window.height();
    
    // Get mouse position in world coordinates (if cursor is in window)
    let mouse_world_pos = window.cursor_position().map(|screen_pos| {
        // Convert screen coordinates to world coordinates
        // Screen: (0,0) is top-left, (width, height) is bottom-right
        // World: (0,0) is center
        Vec2::new(
            screen_pos.x - window_width / 2.0,
            window_height / 2.0 - screen_pos.y,  // Y is inverted
        )
    });
    
    // Snapshot all positions and velocities
    let boid_data: Vec<(Vec3, Vec2)> = query.iter()
        .map(|(_, transform, velocity)| (transform.translation, velocity.0))
        .collect();

    for (i, (_, mut transform, mut velocity)) in query.iter_mut().enumerate() {
        let mut alignment = Vec2::ZERO;
        let mut cohesion = Vec3::ZERO;
        let mut separation = Vec3::ZERO;
        let mut neighbors = 0;

        // Flocking behavior with other boids
        for (j, (other_pos, other_vel)) in boid_data.iter().enumerate() {
            if i == j {
                continue;
            }

            let diff = *other_pos - transform.translation;
            let dist = diff.length();

            if dist < VIEW_RADIUS && dist > 0.0 {
                alignment += *other_vel;
                cohesion += *other_pos;
                separation -= diff / (dist * dist);
                neighbors += 1;
            }
        }

        if neighbors > 0 {
            let n = neighbors as f32;
            alignment = (alignment / n).normalize_or_zero() * ALIGN_WEIGHT;
            cohesion = ((cohesion / n) - transform.translation).normalize_or_zero() * COHESION_WEIGHT;
            separation = separation.normalize_or_zero() * SEPARATION_WEIGHT;
        }
        
        // Calculate distance to each border edge and apply avoidance force
        let mut avoidance = Vec2::ZERO;
        let pos = transform.translation.truncate();

        let left_edge = -window_width / 2.0;
        let right_edge = window_width / 2.0;
        let bottom_edge = -window_height / 2.0;
        let top_edge = window_height / 2.0;
        
        if pos.x - left_edge < WINDOW_AVOIDANCE_DISTANCE {
            let distance = pos.x - left_edge;
            avoidance.x += (1.0_f32 - distance / WINDOW_AVOIDANCE_DISTANCE).max(0.0);
        }
        if right_edge - pos.x < WINDOW_AVOIDANCE_DISTANCE {
            let distance = right_edge - pos.x;
            avoidance.x -= (1.0_f32 - distance / WINDOW_AVOIDANCE_DISTANCE).max(0.0);
        }
        if pos.y - bottom_edge < WINDOW_AVOIDANCE_DISTANCE {
            let distance = pos.y - bottom_edge;
            avoidance.y += (1.0_f32 - distance / WINDOW_AVOIDANCE_DISTANCE).max(0.0);
        }
        if top_edge - pos.y < WINDOW_AVOIDANCE_DISTANCE {
            let distance = top_edge - pos.y;
            avoidance.y -= (1.0_f32 - distance / WINDOW_AVOIDANCE_DISTANCE).max(0.0);
        }
        
        avoidance = avoidance.normalize_or_zero() * WINDOW_AVOIDANCE_WEIGHT;

        // Mouse attraction - steer towards cursor
        let mouse_attraction = if let Some(mouse_pos) = mouse_world_pos {
            let direction = mouse_pos - pos;
            if direction.length() < MOUSE_ATTRACTION_DISTANCE {
                direction.normalize_or_zero() * MOUSE_ATTRACTION_WEIGHT
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        // Combine all forces and update velocity
        velocity.0 = (alignment + cohesion.truncate() + separation.truncate() + avoidance + mouse_attraction).clamp_length_max(MAX_SPEED);

        // Update visual rotation
        transform.rotation = Quat::from_rotation_z(velocity.0.y.atan2(velocity.0.x));
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

// Returns `Some` if `boid` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `boid` hit.
fn boid_collision(boid: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !boid.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(boid.center());
    let offset = boid.center() - closest;
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
